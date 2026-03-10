use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;

/// Send a JSON-RPC message over stdin using newline-delimited JSON transport.
async fn send_message<W: AsyncWriteExt + Unpin>(writer: &mut W, msg: &serde_json::Value) {
    let line = serde_json::to_string(msg).expect("failed to serialize message");
    writer
        .write_all(line.as_bytes())
        .await
        .expect("failed to write message");
    writer
        .write_all(b"\n")
        .await
        .expect("failed to write newline");
    writer.flush().await.expect("failed to flush");
}

/// Read one line from stdout and parse it as JSON.
async fn read_message<R: AsyncBufReadExt + Unpin>(reader: &mut R) -> serde_json::Value {
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .expect("failed to read line");
    serde_json::from_str(line.trim()).expect("failed to parse JSON response")
}

#[tokio::test]
async fn test_initialize_and_list_tools() {
    let binary = env!("CARGO_BIN_EXE_rinda-mcp");

    let mut child = Command::new(binary)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("failed to spawn rinda-mcp binary");

    let mut stdin = child.stdin.take().expect("failed to get stdin");
    let stdout = child.stdout.take().expect("failed to get stdout");
    let mut reader = BufReader::new(stdout);

    // Send initialize request
    let initialize_request = serde_json::json!({
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
    send_message(&mut stdin, &initialize_request).await;

    // Read and verify the initialize response
    let response = read_message(&mut reader).await;

    assert_eq!(response["jsonrpc"], "2.0", "should be JSON-RPC 2.0");
    assert_eq!(response["id"], 1, "should match request id");
    assert!(
        response["error"].is_null(),
        "should not have an error: {:?}",
        response["error"]
    );

    let result = &response["result"];
    assert!(!result.is_null(), "initialize result should not be null");
    assert_eq!(
        result["serverInfo"]["name"], "rinda-mcp",
        "server name should be rinda-mcp"
    );
    assert!(
        !result["capabilities"]["tools"].is_null(),
        "should advertise tools capability"
    );

    // Send initialized notification (no response expected)
    let initialized_notification = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "notifications/initialized",
        "params": {}
    });
    send_message(&mut stdin, &initialized_notification).await;

    // Send tools/list request
    let tools_list_request = serde_json::json!({
        "jsonrpc": "2.0",
        "id": 2,
        "method": "tools/list",
        "params": {}
    });
    send_message(&mut stdin, &tools_list_request).await;

    // Read and verify the tools/list response
    let tools_response = read_message(&mut reader).await;

    assert_eq!(tools_response["jsonrpc"], "2.0", "should be JSON-RPC 2.0");
    assert_eq!(tools_response["id"], 2, "should match request id");
    assert!(
        tools_response["error"].is_null(),
        "should not have an error: {:?}",
        tools_response["error"]
    );

    let tools = &tools_response["result"]["tools"];
    assert!(
        tools.is_array(),
        "tools should be an array, got: {:?}",
        tools
    );
    assert_eq!(
        tools.as_array().unwrap().len(),
        0,
        "tools list should be empty"
    );

    // Close stdin to trigger server shutdown
    drop(stdin);

    // Wait for the process to exit cleanly
    let _ = child.wait().await;
}
