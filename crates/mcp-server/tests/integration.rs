/// Parse SSE-framed body and return the data payloads as parsed JSON values.
/// SSE events are separated by double newlines; each event may have lines
/// like "id: ...", "retry: ...", "data: ...".  We extract only "data:" lines
/// that contain non-empty JSON objects.
fn parse_sse_events(body: &str) -> Vec<serde_json::Value> {
    let mut results = Vec::new();
    for event in body.split("\n\n") {
        for line in event.lines() {
            if let Some(data) = line.strip_prefix("data:") {
                let data = data.trim();
                if data.is_empty() {
                    continue;
                }
                if let Ok(v) = serde_json::from_str::<serde_json::Value>(data) {
                    if !v.is_null() {
                        results.push(v);
                    }
                }
            }
        }
    }
    results
}

/// Spawn the compiled `rinda-mcp` binary listening on a random port.
/// Waits until the server prints its listen address to stderr.
/// Returns (port, child_process).
async fn spawn_binary_server() -> (u16, tokio::process::Child) {
    use tokio::io::{AsyncBufReadExt, BufReader};
    use tokio::process::Command;

    // Pick a random free port by binding and immediately releasing.
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let binary = env!("CARGO_BIN_EXE_rinda-mcp");
    let mut child = Command::new(binary)
        .env("PORT", port.to_string())
        .env("RINDA_API_BASE_URL", "http://localhost:0") // prevent real network calls
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("failed to spawn rinda-mcp binary");

    // Wait until the server prints its listen address to stderr
    let stderr = child.stderr.take().expect("stderr piped");
    let mut reader = BufReader::new(stderr);
    tokio::time::timeout(std::time::Duration::from_secs(10), async {
        let mut line = String::new();
        loop {
            line.clear();
            reader.read_line(&mut line).await.unwrap();
            if line.contains("listening") || line.contains(&port.to_string()) {
                break;
            }
        }
    })
    .await
    .expect("server did not start within 10 seconds");

    // Give it a brief moment to fully bind
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    (port, child)
}

/// Acceptance criteria: GET /health returns 200 {"status": "ok"}
/// (from issue #81: "Add GET /health endpoint")
#[tokio::test]
async fn test_health_endpoint() {
    let (port, mut child) = spawn_binary_server().await;

    let client = reqwest::Client::new();
    let resp = client
        .get(format!("http://127.0.0.1:{port}/health"))
        .send()
        .await
        .expect("health request failed");

    assert_eq!(resp.status(), 200, "health endpoint should return 200");

    let body: serde_json::Value = resp.json().await.expect("health response should be JSON");
    assert_eq!(
        body["status"], "ok",
        "health response should be {{\"status\": \"ok\"}}"
    );

    child.kill().await.ok();
    let _ = child.wait().await;
}

/// Acceptance criteria: MCP server exposes all 14 tools over HTTP transport
/// (from issue #81: "convert MCP server from stdio to remote HTTP transport")
#[tokio::test]
async fn test_initialize_and_list_tools_http() {
    let (port, mut child) = spawn_binary_server().await;

    let client = reqwest::Client::new();
    let base = format!("http://127.0.0.1:{port}");

    // ── 1. Send initialize request ────────────────────────────────────────────
    let init_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "initialize",
        "params": {
            "protocolVersion": "2024-11-05",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        }
    });

    let resp = client
        .post(&base)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&init_body)
        .send()
        .await
        .expect("initialize request failed");

    assert_eq!(
        resp.status(),
        200,
        "initialize should return 200, got: {}",
        resp.status()
    );

    // Extract session ID from response header
    let session_id = resp
        .headers()
        .get("mcp-session-id")
        .expect("mcp-session-id header should be present in initialize response")
        .to_str()
        .unwrap()
        .to_owned();

    let init_text = resp
        .text()
        .await
        .expect("failed to read initialize response");
    let events = parse_sse_events(&init_text);
    assert!(
        !events.is_empty(),
        "initialize response should contain at least one SSE event, got body: {init_text:?}"
    );

    // Find the JSON-RPC response event (id == 1)
    let init_result = events
        .iter()
        .find(|v| v["id"] == 1)
        .expect("initialize response event with id=1 not found");

    assert_eq!(init_result["jsonrpc"], "2.0", "should be JSON-RPC 2.0");
    assert!(
        init_result["error"].is_null(),
        "initialize should not error: {:?}",
        init_result["error"]
    );
    assert_eq!(
        init_result["result"]["serverInfo"]["name"], "rinda-mcp",
        "server name should be rinda-mcp"
    );
    assert!(
        !init_result["result"]["capabilities"]["tools"].is_null(),
        "server should advertise tools capability"
    );

    // ── 2. Send initialized notification ────────────────────────────────────
    let notif_body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });

    let notif_resp = client
        .post(&base)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("mcp-session-id", &session_id)
        .json(&notif_body)
        .send()
        .await
        .expect("initialized notification failed");

    assert!(
        notif_resp.status().is_success(),
        "initialized notification should succeed, got: {}",
        notif_resp.status()
    );

    // ── 3. Send tools/list request ────────────────────────────────────────────
    let list_body = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });

    let list_resp = client
        .post(&base)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .header("mcp-session-id", &session_id)
        .json(&list_body)
        .send()
        .await
        .expect("tools/list request failed");

    assert_eq!(
        list_resp.status(),
        200,
        "tools/list should return 200, got: {}",
        list_resp.status()
    );

    let list_text = list_resp
        .text()
        .await
        .expect("failed to read tools/list response");
    let list_events = parse_sse_events(&list_text);

    // Find the JSON-RPC response event (id == 2)
    let tools_result = list_events
        .iter()
        .find(|v| v["id"] == 2)
        .expect("tools/list response event with id=2 not found");

    assert_eq!(tools_result["jsonrpc"], "2.0", "should be JSON-RPC 2.0");
    assert!(
        tools_result["error"].is_null(),
        "tools/list should not error: {:?}",
        tools_result["error"]
    );

    let tools = &tools_result["result"]["tools"];
    assert!(tools.is_array(), "tools should be an array, got: {tools:?}");
    let tool_list = tools.as_array().unwrap();
    assert_eq!(
        tool_list.len(),
        17,
        "tools list should have 17 tools, got: {}",
        tool_list.len()
    );

    // Verify all 17 expected tool names are present
    let tool_names: Vec<&str> = tool_list
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    let expected_tools = [
        "rinda_auth_status",
        "rinda_buyer_search",
        "rinda_buyer_status",
        "rinda_buyer_results",
        "rinda_buyer_select",
        "rinda_buyer_enrich",
        "rinda_buyer_clarify",
        "rinda_buyer_messages",
        "rinda_campaign_stats",
        "rinda_email_send",
        "rinda_reply_check",
        "rinda_sequence_create",
        "rinda_sequence_list",
        "rinda_sequence_generate",
        "rinda_sequence_add_contact",
        "rinda_order_history",
        "rinda_workspace_list",
    ];

    for expected in &expected_tools {
        assert!(
            tool_names.contains(expected),
            "expected tool '{expected}' not found in list: {tool_names:?}"
        );
    }

    // ── 4. Shutdown ───────────────────────────────────────────────────────────
    child.kill().await.ok();
    let _ = child.wait().await;
}
