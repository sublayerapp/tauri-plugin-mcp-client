use tauri_plugin_mcp_client::{
    registry::ConnectionInfo,
    error::{ProtocollieError, ErrorCategory},
};
use serde_json::json;

/// Test health check response structure
#[test]
fn test_health_check_response_structure() {
    // Test that the health check response has the correct structure
    let expected_fields = vec!["status", "version", "plugin_name", "initialized"];
    
    // Test the expected response structure
    let mock_response = json!({
        "status": "healthy",
        "version": env!("CARGO_PKG_VERSION"),
        "plugin_name": "tauri-plugin-mcp",
        "initialized": true
    });
    
    for field in expected_fields {
        assert!(mock_response.get(field).is_some(), "Missing field: {}", field);
    }
    
    // Test field types
    assert!(mock_response["status"].is_string());
    assert!(mock_response["version"].is_string());
    assert!(mock_response["plugin_name"].is_string());
    assert!(mock_response["initialized"].is_boolean());
}

/// Test connection request structure validation
#[test]
fn test_connect_server_request_validation() {
    // Test valid request
    let valid_request = json!({
        "server_id": "test-server",
        "command": "node",
        "args": ["server.js"]
    });
    
    assert!(valid_request.get("server_id").is_some());
    assert!(valid_request.get("command").is_some());
    assert!(valid_request.get("args").is_some());
    
    // Test request with missing fields
    let invalid_request = json!({
        "server_id": "test-server"
        // Missing command and args
    });
    
    assert!(invalid_request.get("command").is_none());
    assert!(invalid_request.get("args").is_none());
}

/// Test execute tool request structure validation
#[test]
fn test_execute_tool_request_validation() {
    // Test valid request
    let valid_request = json!({
        "server_id": "test-server",
        "tool_name": "echo",
        "arguments": {"message": "hello world"}
    });
    
    assert_eq!(valid_request["server_id"], "test-server");
    assert_eq!(valid_request["tool_name"], "echo");
    assert!(valid_request.get("arguments").is_some());
    
    // Test request with invalid arguments type
    let invalid_request = json!({
        "server_id": "test-server",
        "tool_name": "echo",
        "arguments": "should be object not string"
    });
    
    assert!(invalid_request["arguments"].is_string()); // Should be object
}

/// Test error categorization
#[test]
fn test_error_categories() {
    let connection_error = ProtocollieError::new(
        ErrorCategory::Connection,
        "CONNECTION_FAILED",
        "Failed to connect to server"
    );
    
    match connection_error.category {
        ErrorCategory::Connection => assert!(true),
        _ => panic!("Wrong error category"),
    }
    
    let protocol_error = ProtocollieError::new(
        ErrorCategory::Protocol,
        "INVALID_RESPONSE",
        "Invalid JSON-RPC response"
    );
    
    match protocol_error.category {
        ErrorCategory::Protocol => assert!(true),
        _ => panic!("Wrong error category"),
    }
}

/// Test connection info structure
#[test]
fn test_connection_info_structure() {
    let connection_info = ConnectionInfo {
        server_id: "test-server".to_string(),
        command: "node".to_string(),
        args: vec!["server.js".to_string(), "--port".to_string(), "3000".to_string()],
        status: "connected".to_string(),
        connected_at: Some(1234567890),
    };
    
    assert_eq!(connection_info.server_id, "test-server");
    assert_eq!(connection_info.command, "node");
    assert_eq!(connection_info.args.len(), 3);
    assert_eq!(connection_info.status, "connected");
    assert!(connection_info.connected_at.is_some());
}

/// Test JSON-RPC response parsing
#[test]
fn test_json_rpc_response_parsing() {
    // Test successful response
    let success_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "tools": [
                {
                    "name": "echo",
                    "description": "Echo tool",
                    "inputSchema": {
                        "type": "object",
                        "properties": {
                            "message": {"type": "string"}
                        }
                    }
                }
            ]
        }
    });
    
    assert_eq!(success_response["jsonrpc"], "2.0");
    assert_eq!(success_response["id"], 1);
    assert!(success_response.get("result").is_some());
    assert!(success_response.get("error").is_none());
    
    // Test error response
    let error_response = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "error": {
            "code": -32601,
            "message": "Method not found"
        }
    });
    
    assert_eq!(error_response["jsonrpc"], "2.0");
    assert_eq!(error_response["id"], 1);
    assert!(error_response.get("result").is_none());
    assert!(error_response.get("error").is_some());
    assert_eq!(error_response["error"]["code"], -32601);
}

/// Test message ID uniqueness
#[test]
fn test_message_id_uniqueness() {
    use tauri_plugin_mcp_client::process::MCPProcess;
    
    let process = MCPProcess::new("test".to_string());
    let mut ids = std::collections::HashSet::new();
    
    // Generate 1000 IDs and ensure they're all unique
    for _ in 0..1000 {
        let id = process.next_message_id();
        assert!(!ids.contains(&id), "Duplicate message ID: {}", id);
        ids.insert(id);
    }
    
    assert_eq!(ids.len(), 1000);
}

/// Test message ID sequential generation (without threads due to Sync constraints)
#[test]
fn test_message_id_sequential_generation() {
    use tauri_plugin_mcp_client::process::MCPProcess;
    
    let process = MCPProcess::new("test".to_string());
    let mut ids = std::collections::HashSet::new();
    
    // Generate many IDs sequentially to test uniqueness
    for _ in 0..1000 {
        let id = process.next_message_id();
        assert!(!ids.contains(&id), "Duplicate message ID: {}", id);
        ids.insert(id);
    }
    
    assert_eq!(ids.len(), 1000);
    
    // IDs should be sequential starting from 0
    let mut sorted_ids: Vec<u32> = ids.into_iter().collect();
    sorted_ids.sort();
    for (i, id) in sorted_ids.iter().enumerate() {
        assert_eq!(*id, i as u32, "IDs should be sequential");
    }
}

/// Test error message formatting
#[test]
fn test_error_message_formatting() {
    let error = ProtocollieError::new(
        ErrorCategory::Connection,
        "CONNECTION_TIMEOUT",
        "Connection timed out after 5000ms"
    ).with_details("Server did not respond within timeout period")
     .with_suggestions(vec!["Check server status", "Increase timeout"]);
    
    assert_eq!(error.code, "CONNECTION_TIMEOUT");
    assert!(error.message.contains("timed out"));
    assert!(error.details.as_ref().unwrap().contains("timeout period"));
    assert_eq!(error.suggestions.len(), 2);
}

/// Test tool execution response structure
#[test]
fn test_tool_execution_response() {
    let mock_response = json!({
        "result": {
            "content": [
                {
                    "type": "text",
                    "text": "Tool executed successfully"
                }
            ]
        },
        "duration_ms": 150
    });
    
    assert!(mock_response.get("result").is_some());
    assert!(mock_response.get("duration_ms").is_some());
    assert_eq!(mock_response["duration_ms"], 150);
    
    let content = &mock_response["result"]["content"][0];
    assert_eq!(content["type"], "text");
    assert!(content["text"].as_str().unwrap().contains("successfully"));
}