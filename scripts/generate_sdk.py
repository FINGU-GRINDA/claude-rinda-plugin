#!/usr/bin/env python3
"""Generate rinda-sdk Rust crate from OpenAPI spec."""

import json
import re
import sys
from collections import defaultdict
from pathlib import Path

SPEC_PATH = Path(__file__).parent.parent / "doc" / "openapi.json"
SDK_DIR = Path(__file__).parent.parent / "crates" / "rinda-sdk" / "src"
ENDPOINTS_DIR = SDK_DIR / "endpoints"


def load_spec():
    with open(SPEC_PATH) as f:
        return json.load(f)


def sanitize_module_name(name: str) -> str:
    """Convert tag/path segment to a valid Rust module name."""
    return re.sub(r"[^a-z0-9_]", "_", name.lower().replace("-", "_"))


def to_snake_case(name: str) -> str:
    """Convert operationId or path to snake_case function name."""
    # Remove common prefixes
    name = re.sub(r"^(get|post|put|patch|delete)", "", name)
    # camelCase -> snake_case
    s = re.sub(r"([A-Z])", r"_\1", name).lower()
    s = re.sub(r"[^a-z0-9]", "_", s)
    s = re.sub(r"_+", "_", s).strip("_")
    return s


# Method names reserved by RindaClient impl (client.rs).
RESERVED_NAMES = {"get", "post", "put", "patch", "delete", "new", "default",
                  "with_base_url", "with_access_token", "set_access_token",
                  "http", "base_url", "access_token", "apply_auth",
                  "handle_response", "handle_response_empty"}


def make_fn_name(method: str, path: str, operation_id: str | None) -> str:
    """Generate a unique, descriptive function name.

    Always uses method + path to avoid collisions (GET/PUT/DELETE on same path).
    """
    parts = path.strip("/").split("/")

    # Replace path params with "by_<name>"
    cleaned = []
    for p in parts:
        if not p:
            continue
        if p.startswith("{"):
            cleaned.append("by_" + p.strip("{}").replace("-", "_"))
        else:
            cleaned.append(p.replace("-", "_"))

    if cleaned:
        name = method + "_" + "_".join(cleaned)
    else:
        name = method + "_root"

    name = re.sub(r"_+", "_", name).strip("_")

    # Avoid collisions with client methods
    if name in RESERVED_NAMES:
        name = name + "_endpoint"

    return name


def extract_path_params(path: str) -> list[str]:
    """Extract {paramName} from path."""
    return re.findall(r"\{(\w+)\}", path)


def extract_query_params(details: dict) -> list[dict]:
    """Extract query parameters with name and required flag."""
    params = []
    for p in details.get("parameters", []):
        if p.get("in") == "query":
            params.append(
                {
                    "name": p["name"],
                    "required": p.get("required", False),
                    "schema": p.get("schema", {}),
                }
            )
    return params


def has_request_body(details: dict) -> bool:
    """Check if endpoint accepts a request body."""
    rb = details.get("requestBody")
    if not rb:
        return False
    # POST/PUT/PATCH typically have bodies
    return True


def rust_type_for_schema(schema: dict) -> str:
    """Map OpenAPI schema to Rust type. Fallback to Value."""
    t = schema.get("type")
    if t == "string":
        return "String"
    if t == "integer":
        return "i64"
    if t == "number":
        return "f64"
    if t == "boolean":
        return "bool"
    if t == "array":
        inner = rust_type_for_schema(schema.get("items", {}))
        return f"Vec<{inner}>"
    return "serde_json::Value"


def group_endpoints(spec: dict) -> dict[str, list]:
    """Group endpoints by module name."""
    groups = defaultdict(list)
    paths = spec.get("paths", {})

    for path, methods in paths.items():
        for method, details in methods.items():
            if method not in ("get", "post", "put", "patch", "delete"):
                continue
            tags = details.get("tags", [])
            if tags:
                group = sanitize_module_name(tags[0])
            else:
                parts = path.strip("/").split("/")
                if len(parts) >= 3 and parts[0] == "api" and parts[1] == "v1":
                    group = sanitize_module_name(parts[2])
                elif len(parts) >= 2 and parts[0] == "api":
                    group = sanitize_module_name(parts[1])
                else:
                    group = "misc"
            groups[group].append((method, path, details))

    return dict(groups)


def generate_endpoint_module(module_name: str, endpoints: list) -> str:
    """Generate a Rust module file for a group of endpoints."""
    lines = [
        f"//! Endpoints for `{module_name}`.",
        "//!",
        "//! Auto-generated from OpenAPI spec. Do not edit manually.",
        "",
        "use crate::client::RindaClient;",
        "use crate::error::Result;",
        "",
        "impl RindaClient {",
    ]

    seen_names = set()

    for method, path, details in endpoints:
        op_id = details.get("operationId")
        summary = details.get("summary", "")
        description = details.get("description", "")

        fn_name = make_fn_name(method, path, op_id)

        # Deduplicate function names
        base_name = fn_name
        counter = 2
        while fn_name in seen_names:
            fn_name = f"{base_name}_{counter}"
            counter += 1
        seen_names.add(fn_name)

        # Determine parameters
        path_params = extract_path_params(path)
        query_params = extract_query_params(details)
        needs_body = has_request_body(details) and method in (
            "post",
            "put",
            "patch",
        )

        # Build doc comment
        doc_line = f"    /// `{method.upper()} {path}`"
        if summary:
            doc_line += f" — {summary}"
        lines.append(doc_line)
        if description:
            for desc_line in description.split("\n"):
                lines.append(f"    /// {desc_line.strip()}")

        # Build function signature
        params = []
        for pp in path_params:
            param_name = re.sub(r"[^a-z0-9_]", "_", pp.lower())
            # Avoid Rust keyword collisions
            if param_name in ("type", "self", "ref", "match"):
                param_name = f"r#{param_name}"
            params.append(f"{param_name}: &str")

        if query_params:
            params.append("query: &[(&str, &str)]")

        if needs_body:
            params.append("body: &serde_json::Value")

        params_str = ", ".join(params)
        if params_str:
            params_str = ", " + params_str

        lines.append(
            f"    pub async fn {fn_name}(&self{params_str}) -> Result<serde_json::Value> {{"
        )

        # Build path string with interpolation
        rust_path = path
        for pp in path_params:
            param_name = re.sub(r"[^a-z0-9_]", "_", pp.lower())
            if param_name in ("type", "self", "ref", "match"):
                param_name = f"r#{param_name}"
            rust_path = rust_path.replace("{" + pp + "}", f"{{{param_name}}}")

        if path_params:
            lines.append(f'        let path = format!("{rust_path}");')
            path_ref = "&path"
        else:
            path_ref = f'"{path}"'

        # Build the request
        method_fn = method  # get, post, put, patch, delete
        lines.append(f"        let req = self.{method_fn}({path_ref});")

        # Add query params
        if query_params:
            lines.append("        let req = req.query(&query);")

        # Add body
        if needs_body:
            lines.append("        let req = req.json(body);")

        lines.append("        let resp = req.send().await?;")
        lines.append("        Self::handle_response(resp).await")
        lines.append("    }")
        lines.append("")

    lines.append("}")
    lines.append("")
    return "\n".join(lines)


def generate_mod_rs(module_names: list[str]) -> str:
    """Generate endpoints/mod.rs."""
    lines = [
        "//! API endpoint modules.",
        "//!",
        "//! Auto-generated from OpenAPI spec. Do not edit manually.",
        "",
    ]
    for name in sorted(module_names):
        lines.append(f"pub mod {name};")
    lines.append("")
    return "\n".join(lines)


def generate_lib_rs(module_names: list[str]) -> str:
    """Generate lib.rs."""
    return """//! Rinda SDK — auto-generated API client.
//!
//! Generated from `doc/openapi.json`. Do not edit manually.

pub mod client;
pub mod endpoints;
pub mod error;
"""


def main():
    spec = load_spec()
    groups = group_endpoints(spec)

    # Clean existing endpoints
    ENDPOINTS_DIR.mkdir(parents=True, exist_ok=True)
    for f in ENDPOINTS_DIR.glob("*.rs"):
        f.unlink()

    total_methods = 0
    module_names = []

    for module_name, endpoints in sorted(groups.items()):
        code = generate_endpoint_module(module_name, endpoints)
        out_path = ENDPOINTS_DIR / f"{module_name}.rs"
        out_path.write_text(code)
        module_names.append(module_name)
        total_methods += len(endpoints)
        print(f"  {module_name}.rs — {len(endpoints)} methods")

    # Write mod.rs
    mod_code = generate_mod_rs(module_names)
    (ENDPOINTS_DIR / "mod.rs").write_text(mod_code)

    # Write lib.rs
    lib_code = generate_lib_rs(module_names)
    (SDK_DIR / "lib.rs").write_text(lib_code)

    print(f"\nGenerated {len(module_names)} modules with {total_methods} methods total.")
    print(f"Output: {ENDPOINTS_DIR}")


if __name__ == "__main__":
    main()
