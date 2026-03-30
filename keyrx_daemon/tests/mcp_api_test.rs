//! MCP endpoint integration tests.
//!
//! Tests the Streamable HTTP MCP endpoint at `/mcp` by sending JSON-RPC 2.0
//! requests and verifying responses follow the MCP specification.
//!
//! Run with: `cargo test -p keyrx_daemon --test mcp_api_test -- --test-threads=1`

mod common;

use common::test_app::TestApp;
use serde_json::{json, Value};
use serial_test::serial;

/// Send a JSON-RPC 2.0 request to the MCP endpoint with proper Accept headers.
async fn mcp_request(app: &TestApp, method: &str, params: Value) -> Value {
    let body = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });

    let client = reqwest::Client::new();
    let url = format!("{}/mcp", app.base_url);
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .header("Accept", "application/json, text/event-stream")
        .json(&body)
        .send()
        .await
        .expect("Failed to send MCP request");

    let status = response.status();
    let text = response.text().await.unwrap_or_default();

    assert!(
        status.is_success() || status.as_u16() == 202,
        "MCP request '{}' failed: status={}, body={}",
        method,
        status,
        text
    );

    // Parse SSE or JSON response
    if text.starts_with("data:") || text.contains("\ndata:") {
        // SSE format: extract JSON from data lines
        parse_sse_response(&text)
    } else {
        serde_json::from_str(&text).unwrap_or_else(|_| json!({"raw": text}))
    }
}

/// Parse SSE response format, extracting the last data event.
fn parse_sse_response(text: &str) -> Value {
    let mut last_data = None;
    for line in text.lines() {
        if let Some(data) = line.strip_prefix("data:") {
            let data = data.trim();
            if !data.is_empty() {
                last_data = serde_json::from_str(data).ok();
            }
        }
    }
    last_data.unwrap_or_else(|| json!({"raw": text}))
}

#[tokio::test]
#[serial]
async fn test_mcp_initialize() {
    let app = TestApp::new().await;

    let result = mcp_request(
        &app,
        "initialize",
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {
                "name": "test-client",
                "version": "0.1.0"
            }
        }),
    )
    .await;

    // Should have a result with server info
    let server_info = &result["result"]["serverInfo"];
    assert!(
        server_info.is_object(),
        "Expected serverInfo in initialize result, got: {}",
        result
    );
}

#[tokio::test]
#[serial]
async fn test_mcp_tools_list() {
    let app = TestApp::new().await;

    // Initialize first
    let _ = mcp_request(
        &app,
        "initialize",
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0.1"}
        }),
    )
    .await;

    // List tools
    let result = mcp_request(&app, "tools/list", json!({})).await;

    let tools = &result["result"]["tools"];
    assert!(tools.is_array(), "Expected tools array, got: {}", result);

    let tools_array = tools.as_array().unwrap();
    assert!(
        tools_array.len() >= 12,
        "Expected at least 12 tools, got {}",
        tools_array.len()
    );

    // Verify some expected tool names
    let tool_names: Vec<&str> = tools_array
        .iter()
        .filter_map(|t| t["name"].as_str())
        .collect();

    assert!(
        tool_names.contains(&"keyrx_list_profiles"),
        "Missing keyrx_list_profiles tool. Tools: {:?}",
        tool_names
    );
    assert!(
        tool_names.contains(&"keyrx_get_status"),
        "Missing keyrx_get_status tool"
    );
    assert!(
        tool_names.contains(&"keyrx_get_diagnostics"),
        "Missing keyrx_get_diagnostics tool"
    );
}

#[tokio::test]
#[serial]
async fn test_mcp_call_list_profiles() {
    let app = TestApp::new().await;

    // Initialize
    let _ = mcp_request(
        &app,
        "initialize",
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0.1"}
        }),
    )
    .await;

    // Call list_profiles tool
    let result = mcp_request(
        &app,
        "tools/call",
        json!({
            "name": "keyrx_list_profiles",
            "arguments": {}
        }),
    )
    .await;

    // Result should have content array
    let content = &result["result"]["content"];
    assert!(
        content.is_array(),
        "Expected content array in tool result, got: {}",
        result
    );
}

#[tokio::test]
#[serial]
async fn test_mcp_call_get_diagnostics() {
    let app = TestApp::new().await;

    // Initialize
    let _ = mcp_request(
        &app,
        "initialize",
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0.1"}
        }),
    )
    .await;

    // Call get_diagnostics tool
    let result = mcp_request(
        &app,
        "tools/call",
        json!({
            "name": "keyrx_get_diagnostics",
            "arguments": {}
        }),
    )
    .await;

    let content = &result["result"]["content"];
    assert!(
        content.is_array(),
        "Expected content array, got: {}",
        result
    );

    // Parse the text content — should contain version info
    if let Some(text) = content[0]["text"].as_str() {
        let diag: Value = serde_json::from_str(text).unwrap_or_default();
        assert!(
            diag["version"].is_string(),
            "Expected version in diagnostics"
        );
        assert!(
            diag["platform"].is_string(),
            "Expected platform in diagnostics"
        );
    }
}

#[tokio::test]
#[serial]
async fn test_mcp_call_get_status() {
    let app = TestApp::new().await;

    // Initialize
    let _ = mcp_request(
        &app,
        "initialize",
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0.1"}
        }),
    )
    .await;

    let result = mcp_request(
        &app,
        "tools/call",
        json!({
            "name": "keyrx_get_status",
            "arguments": {}
        }),
    )
    .await;

    let content = &result["result"]["content"];
    assert!(
        content.is_array(),
        "Expected content array, got: {}",
        result
    );
}

#[tokio::test]
#[serial]
async fn test_mcp_call_create_and_get_profile() {
    let app = TestApp::new().await;

    // Initialize
    let _ = mcp_request(
        &app,
        "initialize",
        json!({
            "protocolVersion": "2025-03-26",
            "capabilities": {},
            "clientInfo": {"name": "test", "version": "0.1"}
        }),
    )
    .await;

    // Create a profile
    let create_result = mcp_request(
        &app,
        "tools/call",
        json!({
            "name": "keyrx_create_profile",
            "arguments": {"name": "mcp-test", "template": "blank"}
        }),
    )
    .await;

    let content = &create_result["result"]["content"];
    assert!(content.is_array(), "Create failed: {}", create_result);

    // Get the profile config
    let config_result = mcp_request(
        &app,
        "tools/call",
        json!({
            "name": "keyrx_get_profile_config",
            "arguments": {"name": "mcp-test"}
        }),
    )
    .await;

    let content = &config_result["result"]["content"];
    assert!(content.is_array(), "Get config failed: {}", config_result);
}
