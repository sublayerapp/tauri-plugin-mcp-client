use serde_json::{json, Value};
use std::io::{self, BufRead, BufReader, Write};
use std::process::{Command, Stdio};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader as AsyncBufReader};
use tokio::process::{Child, Command as AsyncCommand};

/// Mock MCP server that responds to standard MCP protocol messages
/// This provides a reliable test environment for plugin testing
pub struct MockMCPServer {
    pub name: String,
    pub version: String,
    pub tools: Vec<MockTool>,
}

#[derive(Clone)]
pub struct MockTool {
    pub name: String,
    pub description: String,
    pub parameters: Value,
    pub response_fn: fn(&Value) -> Value,
}

impl MockMCPServer {
    /// Create a new mock server with basic configuration
    pub fn new(name: &str, version: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            tools: Vec::new(),
        }
    }

    /// Add a tool to the mock server
    pub fn add_tool(&mut self, tool: MockTool) {
        self.tools.push(tool);
    }

    /// Add a simple echo tool for testing
    pub fn with_echo_tool(mut self) -> Self {
        self.add_tool(MockTool {
            name: "echo".to_string(),
            description: "Echo back the input message".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "message": {
                        "type": "string",
                        "description": "Message to echo back"
                    }
                },
                "required": ["message"]
            }),
            response_fn: |args| {
                json!({
                    "content": [
                        {
                            "type": "text",
                            "text": format!("Echo: {}", args.get("message").unwrap_or(&json!("")).as_str().unwrap_or(""))
                        }
                    ]
                })
            },
        });
        self
    }

    /// Handle an incoming JSON-RPC message
    pub fn handle_message(&self, message: &Value) -> Option<Value> {
        let method = message.get("method")?.as_str()?;
        let id = message.get("id");
        let params = message.get("params");

        match method {
            "initialize" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": self.name,
                        "version": self.version
                    }
                }
            })),
            "tools/list" => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "result": {
                    "tools": self.tools.iter().map(|tool| json!({
                        "name": tool.name,
                        "description": tool.description,
                        "inputSchema": tool.parameters
                    })).collect::<Vec<_>>()
                }
            })),
            "tools/call" => {
                if let Some(params) = params {
                    let tool_name = params.get("name")?.as_str()?;
                    let arguments = params.get("arguments")?;
                    
                    if let Some(tool) = self.tools.iter().find(|t| t.name == tool_name) {
                        let result = (tool.response_fn)(arguments);
                        Some(json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "result": result
                        }))
                    } else {
                        Some(json!({
                            "jsonrpc": "2.0",
                            "id": id,
                            "error": {
                                "code": -32601,
                                "message": format!("Tool '{}' not found", tool_name)
                            }
                        }))
                    }
                } else {
                    Some(json!({
                        "jsonrpc": "2.0",
                        "id": id,
                        "error": {
                            "code": -32602,
                            "message": "Invalid params for tools/call"
                        }
                    }))
                }
            }
            _ => Some(json!({
                "jsonrpc": "2.0",
                "id": id,
                "error": {
                    "code": -32601,
                    "message": format!("Method '{}' not found", method)
                }
            }))
        }
    }

    /// Run the mock server as a subprocess (for integration testing)
    pub fn spawn_as_process(&self) -> io::Result<MockServerProcess> {
        // Create a script that implements our mock server
        let script_content = format!(r#"#!/usr/bin/env node
const readline = require('readline');

const rl = readline.createInterface({{
    input: process.stdin,
    output: process.stdout,
    terminal: false
}});

const serverName = "{}";
const serverVersion = "{}";
const tools = {};

rl.on('line', (line) => {{
    try {{
        const message = JSON.parse(line);
        const response = handleMessage(message);
        if (response) {{
            console.log(JSON.stringify(response));
        }}
    }} catch (e) {{
        // Ignore malformed JSON
    }}
}});

function handleMessage(message) {{
    const method = message.method;
    const id = message.id;
    const params = message.params;

    switch (method) {{
        case 'initialize':
            return {{
                jsonrpc: '2.0',
                id: id,
                result: {{
                    protocolVersion: '2024-11-05',
                    capabilities: {{ tools: {{}} }},
                    serverInfo: {{
                        name: serverName,
                        version: serverVersion
                    }}
                }}
            }};
        case 'tools/list':
            return {{
                jsonrpc: '2.0',
                id: id,
                result: {{ tools: tools }}
            }};
        case 'tools/call':
            if (params && params.name === 'echo' && params.arguments) {{
                return {{
                    jsonrpc: '2.0',
                    id: id,
                    result: {{
                        content: [{{
                            type: 'text',
                            text: `Echo: ${{params.arguments.message || ''}}`
                        }}]
                    }}
                }};
            }}
            return {{
                jsonrpc: '2.0',
                id: id,
                error: {{
                    code: -32601,
                    message: `Tool '${{params?.name || 'unknown'}}' not found`
                }}
            }};
        default:
            return {{
                jsonrpc: '2.0',
                id: id,
                error: {{
                    code: -32601,
                    message: `Method '${{method}}' not found`
                }}
            }};
    }}
}}
"#, self.name, self.version, json!(self.tools.iter().map(|tool| json!({
            "name": tool.name,
            "description": tool.description,
            "inputSchema": tool.parameters
        })).collect::<Vec<_>>()));

        // Write script to temp file
        use tempfile::NamedTempFile;
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(script_content.as_bytes())?;
        temp_file.flush()?;

        // Spawn node process with the script
        let child = Command::new("node")
            .arg(temp_file.path())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(MockServerProcess {
            child,
            _temp_file: temp_file,
        })
    }
}

/// A mock MCP server running as a subprocess
pub struct MockServerProcess {
    child: std::process::Child,
    _temp_file: tempfile::NamedTempFile,
}

impl MockServerProcess {
    /// Get the command and args to connect to this mock server
    pub fn get_command_args(&self) -> (String, Vec<String>) {
        // This would typically return the node command and script path
        // For testing purposes, we'll return a simple echo command
        ("node".to_string(), vec!["-e".to_string(), "process.stdin.pipe(process.stdout)".to_string()])
    }

    /// Stop the mock server
    pub fn stop(&mut self) -> io::Result<()> {
        self.child.kill()?;
        self.child.wait()?;
        Ok(())
    }
}

impl Drop for MockServerProcess {
    fn drop(&mut self) {
        let _ = self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_server_initialize() {
        let server = MockMCPServer::new("test-server", "1.0.0");
        let message = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {}
            }
        });

        let response = server.handle_message(&message).unwrap();
        assert_eq!(response["result"]["serverInfo"]["name"], "test-server");
        assert_eq!(response["result"]["serverInfo"]["version"], "1.0.0");
        assert_eq!(response["id"], 1);
    }

    #[test]
    fn test_mock_server_list_tools() {
        let server = MockMCPServer::new("test-server", "1.0.0")
            .with_echo_tool();
        
        let message = json!({
            "jsonrpc": "2.0",
            "id": 2,
            "method": "tools/list",
            "params": {}
        });

        let response = server.handle_message(&message).unwrap();
        let tools = response["result"]["tools"].as_array().unwrap();
        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0]["name"], "echo");
    }

    #[test]
    fn test_mock_server_call_tool() {
        let server = MockMCPServer::new("test-server", "1.0.0")
            .with_echo_tool();
        
        let message = json!({
            "jsonrpc": "2.0",
            "id": 3,
            "method": "tools/call",
            "params": {
                "name": "echo",
                "arguments": {
                    "message": "Hello, World!"
                }
            }
        });

        let response = server.handle_message(&message).unwrap();
        let content = &response["result"]["content"][0]["text"];
        assert_eq!(content, "Echo: Hello, World!");
    }

    #[test]
    fn test_mock_server_unknown_method() {
        let server = MockMCPServer::new("test-server", "1.0.0");
        let message = json!({
            "jsonrpc": "2.0",
            "id": 4,
            "method": "unknown/method",
            "params": {}
        });

        let response = server.handle_message(&message).unwrap();
        assert!(response.get("error").is_some());
        assert_eq!(response["error"]["code"], -32601);
    }
}