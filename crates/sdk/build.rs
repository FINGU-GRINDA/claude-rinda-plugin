use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // Tell cargo to rerun if the spec changes.
    println!("cargo:rerun-if-changed=../../doc/openapi-patched.json");

    let out_dir = env::var("OUT_DIR").expect("OUT_DIR not set");
    let dest = Path::new(&out_dir).join("codegen.rs");

    // Read and parse the OpenAPI spec.
    let spec_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../doc/openapi-patched.json");
    let spec_str = fs::read_to_string(&spec_path)
        .unwrap_or_else(|e| panic!("Failed to read openapi-patched.json: {e}"));
    let mut spec: serde_json::Value = serde_json::from_str(&spec_str)
        .unwrap_or_else(|e| panic!("Failed to parse openapi-patched.json as JSON: {e}"));

    // Patch the spec for progenitor compatibility.
    // Save securitySchemes before patching (they use different type values like "http").
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
        .unwrap_or_else(|e| panic!("Progenitor code generation failed: {e}"));

    let ast = syn::parse2(tokens).unwrap_or_else(|e| panic!("Syn parse error: {e}"));
    let content = prettyplease::unparse(&ast);

    fs::write(&dest, content).unwrap_or_else(|e| panic!("Failed to write codegen.rs: {e}"));
}

/// Apply compatibility patches to the spec so progenitor can process it.
fn patch_spec(spec: &mut serde_json::Value) {
    // First: fix multi-content-type requestBodies (progenitor supports only one).
    if let Some(paths) = spec.get_mut("paths").and_then(|v| v.as_object_mut()) {
        for (_path, methods) in paths.iter_mut() {
            if let Some(methods_obj) = methods.as_object_mut() {
                for (_method, op) in methods_obj.iter_mut() {
                    if let Some(rb) = op.get_mut("requestBody") {
                        fix_multi_content(rb);
                    }
                }
            }
        }
    }

    // Second: recursively remove null types from the entire spec.
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
                // Treat unknown scalar types as "string".
                obj.insert(
                    "type".to_string(),
                    serde_json::Value::String("string".to_string()),
                );
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
