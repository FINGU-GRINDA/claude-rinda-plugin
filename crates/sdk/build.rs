use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo to rerun if the spec changes.
    println!("cargo:rerun-if-changed=../../doc/openapi.json");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("codegen.rs");

    // Read and parse the OpenAPI spec.
    let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../doc/openapi.json");
    let spec_str = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("Failed to read openapi.json: {e}"));
    let mut spec: serde_json::Value =
        serde_json::from_str(&spec_str).unwrap_or_else(|e| panic!("Failed to parse JSON: {e}"));

    // Save securitySchemes before patching (they use non-standard type values like "http").
    let security_schemes = spec
        .get("components")
        .and_then(|c| c.get("securitySchemes"))
        .cloned();

    patch_spec(&mut spec);

    // Restore securitySchemes (patch_spec may have corrupted "type": "http").
    if let Some(ss) = security_schemes
        && let Some(components) = spec.get_mut("components").and_then(|v| v.as_object_mut())
    {
        components.insert("securitySchemes".to_string(), ss);
    }

    // Re-serialize to string for openapiv3 parsing.
    let patched_str = serde_json::to_string(&spec).expect("Failed to re-serialize patched spec");

    let spec: openapiv3::OpenAPI = serde_json::from_str(&patched_str)
        .unwrap_or_else(|e| panic!("Failed to parse patched spec as OpenAPI: {e}"));

    // Generate client code using progenitor.
    let mut settings = progenitor::GenerationSettings::default();
    settings.with_derive("Debug");

    let mut generator = progenitor::Generator::new(&settings);
    let tokens = generator
        .generate_tokens(&spec)
        .unwrap_or_else(|e| panic!("Progenitor code generation failed: {e:?}"));

    let ast = syn::parse2(tokens).unwrap_or_else(|e| panic!("Syn parse error: {e}"));
    let content = prettyplease::unparse(&ast);

    fs::write(&dest, content).unwrap_or_else(|e| panic!("Failed to write codegen.rs: {e}"));
}

const FREEFORM_SCHEMA: &str = r#"{"type":"object","additionalProperties":true}"#;

/// Apply compatibility patches to the spec so progenitor can process it.
fn patch_spec(spec: &mut serde_json::Value) {
    let freeform: serde_json::Value = serde_json::from_str(FREEFORM_SCHEMA).unwrap();
    let json_content = serde_json::json!({
        "application/json": { "schema": freeform }
    });

    if let Some(paths) = spec.get_mut("paths").and_then(|v| v.as_object_mut()) {
        for (_path, methods) in paths.iter_mut() {
            if let Some(methods_obj) = methods.as_object_mut() {
                for (method, op) in methods_obj.iter_mut() {
                    if !["get", "post", "put", "patch", "delete"].contains(&method.as_str()) {
                        continue;
                    }

                    // Fix empty requestBody content.
                    if let Some(rb) = op.get_mut("requestBody") {
                        let content = rb.get("content").and_then(|c| c.as_object());
                        if content.is_none_or(|c| c.is_empty()) {
                            rb.as_object_mut()
                                .unwrap()
                                .insert("content".to_string(), json_content.clone());
                        }
                        // If multiple content types, keep only application/json.
                        fix_multi_content(rb);
                    }

                    // Fix missing responses.
                    let responses = op.get("responses").and_then(|r| r.as_object());
                    if responses.is_none_or(|r| r.is_empty()) {
                        op.as_object_mut().unwrap().insert(
                            "responses".to_string(),
                            serde_json::json!({
                                "200": {
                                    "description": "Successful response",
                                    "content": json_content.clone()
                                }
                            }),
                        );
                    } else if let Some(responses) =
                        op.get_mut("responses").and_then(|r| r.as_object_mut())
                    {
                        // Fix responses with missing content.
                        for (_code, resp) in responses.iter_mut() {
                            let content = resp.get("content").and_then(|c| c.as_object());
                            if content.is_none_or(|c| c.is_empty()) {
                                resp.as_object_mut()
                                    .unwrap()
                                    .insert("content".to_string(), json_content.clone());
                            }
                        }
                    }
                }
            }
        }
    }

    // Recursively remove null types and normalize non-standard type names.
    remove_null_types(spec);
}

/// If a requestBody has multiple content types, keep only application/json.
fn fix_multi_content(rb: &mut serde_json::Value) {
    if let Some(content) = rb.get_mut("content").and_then(|v| v.as_object_mut()) {
        let keys: Vec<String> = content.keys().cloned().collect();
        if keys.len() > 1 {
            let keep = if keys.contains(&"application/json".to_string()) {
                "application/json".to_string()
            } else {
                keys[0].clone()
            };
            let kept = content.remove(&keep);
            content.clear();
            if let Some(v) = kept {
                content.insert(keep, v);
            }
        }
    }
}

/// Recursively:
///   - Remove `{"type":"null"}` entries from anyOf/oneOf/allOf arrays.
///   - Replace non-standard types (e.g. "Date") with "string".
fn remove_null_types(val: &mut serde_json::Value) {
    match val {
        serde_json::Value::Object(obj) => {
            // Normalize non-standard "type" values to standard OpenAPI types.
            const STANDARD_TYPES: &[&str] = &[
                "string", "number", "integer", "boolean", "array", "object", "null",
            ];
            if let Some(t) = obj.get("type").and_then(|v| v.as_str())
                && !STANDARD_TYPES.contains(&t)
            {
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("string".to_string()),
                );
            }

            // Remove default values that don't match the declared type.
            if let Some(typ) = obj.get("type").and_then(|v| v.as_str()).map(String::from) {
                let bad_default = obj.get("default").is_some_and(|d| match typ.as_str() {
                    "string" => !d.is_string(),
                    "number" | "integer" => !d.is_number(),
                    "boolean" => !d.is_boolean(),
                    "array" => !d.is_array(),
                    "object" => !d.is_object(),
                    _ => false,
                });
                if bad_default {
                    obj.remove("default");
                }
            }

            for key in ["anyOf", "oneOf", "allOf"] {
                if let Some(arr) = obj.get_mut(key).and_then(|v| v.as_array_mut()) {
                    arr.retain(|item| item.get("type").and_then(|t| t.as_str()) != Some("null"));
                }
            }
            for (_k, v) in obj.iter_mut() {
                remove_null_types(v);
            }
        }
        serde_json::Value::Array(arr) => {
            for v in arr.iter_mut() {
                remove_null_types(v);
            }
        }
        _ => {}
    }
}
