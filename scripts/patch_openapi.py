#!/usr/bin/env python3
"""Patch OpenAPI spec so openapi-generator can produce usable Rust code.

Fixes:
1. Empty requestBody content -> adds application/json with freeform object
2. Missing responses -> adds generic 200 with freeform JSON
3. Missing response content -> adds application/json with freeform object
"""

import json
from pathlib import Path

SPEC_PATH = Path(__file__).parent.parent / "doc" / "openapi.json"
OUT_PATH = Path(__file__).parent.parent / "doc" / "openapi-patched.json"

FREEFORM_SCHEMA = {"type": "object", "additionalProperties": True}
JSON_CONTENT = {"application/json": {"schema": FREEFORM_SCHEMA}}


def patch():
    with open(SPEC_PATH) as f:
        spec = json.load(f)

    stats = {"patched_request_body": 0, "patched_responses": 0, "patched_response_content": 0}

    for path, methods in spec.get("paths", {}).items():
        for method, details in methods.items():
            if method not in ("get", "post", "put", "patch", "delete"):
                continue

            # Fix empty requestBody content
            rb = details.get("requestBody")
            if rb:
                content = rb.get("content", {})
                if not content:
                    rb["content"] = JSON_CONTENT
                    stats["patched_request_body"] += 1

            # Fix missing responses
            if "responses" not in details or not details["responses"]:
                details["responses"] = {
                    "200": {
                        "description": "Successful response",
                        "content": JSON_CONTENT,
                    }
                }
                stats["patched_responses"] += 1
            else:
                # Fix responses that exist but have no content
                for code, resp in details["responses"].items():
                    if "content" not in resp or not resp["content"]:
                        resp["content"] = JSON_CONTENT
                        stats["patched_response_content"] += 1

    with open(OUT_PATH, "w") as f:
        json.dump(spec, f, indent=2)

    print(f"Patched spec written to {OUT_PATH}")
    for k, v in stats.items():
        print(f"  {k}: {v}")


if __name__ == "__main__":
    patch()
