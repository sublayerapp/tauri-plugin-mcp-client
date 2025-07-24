use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};
use crate::process::MCPProcess;
use crate::error::MCPClientError;
use tauri::{AppHandle, Emitter, Runtime};

/// Event types for real-time MCP connection updates
pub const EVENT_CONNECTION_CHANGED: &str = "mcp://connection-changed";
pub const EVENT_SERVER_CONNECTED: &str = "mcp://server-connected";
pub const EVENT_SERVER_DISCONNECTED: &str = "mcp://server-disconnected";
pub const EVENT_PROCESS_ERROR: &str = "mcp://process-error";

/// Event payload for connection status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionEvent {
    pub server_id: String,
    pub status: String, // "connected", "disconnected", "error"
    pub reason: Option<String>,
    pub timestamp: u64,
    pub command: Option<String>,
    pub args: Option<Vec<String>>,
}

/// Connection status information for a single MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub server_id: String,
    pub command: String,
    pub args: Vec<String>,
    pub status: String,
    pub connected_at: Option<u64>, // Unix timestamp
}

/// Plugin-specific connection registry to track MCP server connections
/// This runs independently from any main MCP system
pub struct ConnectionRegistry<R: Runtime = tauri::Wry> {
    connections: Arc<Mutex<HashMap<String, ConnectionInfo>>>,
    processes: Arc<Mutex<HashMap<String, MCPProcess>>>,
    app_handle: Option<AppHandle<R>>,
}

impl<R: Runtime> ConnectionRegistry<R> {
    /// Create a new empty connection registry
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            processes: Arc::new(Mutex::new(HashMap::new())),
            app_handle: None,
        }
    }

    /// Set the app handle for event emission
    pub fn set_app_handle(&mut self, app_handle: AppHandle<R>) {
        self.app_handle = Some(app_handle);
    }

    /// Emit a connection event if app handle is available
    fn emit_connection_event(&self, event: ConnectionEvent) {
        if let Some(ref app_handle) = self.app_handle {
            eprintln!("DEBUG: About to emit connection event: {:?}", event);
            if let Err(e) = app_handle.emit(EVENT_CONNECTION_CHANGED, &event) {
                eprintln!("DEBUG: Failed to emit connection event: {}", e);
            } else {
                eprintln!("DEBUG: Successfully emitted connection event: {:?}", event);
            }
        } else {
            eprintln!("DEBUG: No app handle available, cannot emit event: {:?}", event);
        }
    }

    /// Get all current connection statuses
    pub fn get_connection_statuses(&self) -> Result<Vec<ConnectionInfo>, String> {
        let connections = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        
        Ok(connections.values().cloned().collect())
    }

    /// Connect to an MCP server through the plugin
    pub async fn connect_server(&self, server_id: String, command: String, args: Vec<String>) -> Result<(), MCPClientError> {
        eprintln!("DEBUG: Plugin connect_server called for {} with command: {} {:?}", server_id, command, args);

        // Stop existing process if any (silently, without emitting events)
        self.disconnect_server_silent(&server_id).await?;

        // Create new MCPProcess
        let mut process = MCPProcess::new(server_id.clone());
        
        // Start the process
        match process.start(&command, &args).await {
            Ok(()) => {
                // Initialize MCP connection
                process.send_initialize()?;
                
                // Store the process
                {
                    let mut processes = self.processes.lock()
                        .map_err(|e| MCPClientError::system_error(&format!("Failed to lock processes: {}", e)))?;
                    processes.insert(server_id.clone(), process);
                }

                // Store connection info
                let connection_info = ConnectionInfo {
                    server_id: server_id.clone(),
                    command: command.clone(),
                    args: args.clone(),
                    status: "connected".to_string(),
                    connected_at: Some(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs()),
                };

                {
                    let mut connections = self.connections.lock()
                        .map_err(|e| MCPClientError::system_error(&format!("Failed to lock connections: {}", e)))?;
                    connections.insert(server_id.clone(), connection_info);
                }

                // Emit connection event
                let event = ConnectionEvent {
                    server_id: server_id.clone(),
                    status: "connected".to_string(),
                    reason: None,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    command: Some(command.clone()),
                    args: Some(args.clone()),
                };
                self.emit_connection_event(event);

                eprintln!("DEBUG: Plugin successfully connected to server {}", server_id);
                Ok(())
            }
            Err(e) => {
                eprintln!("DEBUG: Plugin failed to connect to server {}: {}", server_id, e);
                Err(e)
            }
        }
    }

    /// Disconnect from an MCP server silently (no events)
    async fn disconnect_server_silent(&self, server_id: &str) -> Result<(), MCPClientError> {
        eprintln!("DEBUG: Plugin disconnect_server_silent called for {}", server_id);

        // Remove and stop the process
        {
            let mut processes = self.processes.lock()
                .map_err(|e| MCPClientError::system_error(&format!("Failed to lock processes: {}", e)))?;
            
            if let Some(mut process) = processes.remove(server_id) {
                process.stop();
                eprintln!("DEBUG: Plugin silently stopped process for server {}", server_id);
            }
        }

        // Remove connection info
        {
            let mut connections = self.connections.lock()
                .map_err(|e| MCPClientError::system_error(&format!("Failed to lock connections: {}", e)))?;
            connections.remove(server_id);
        }

        // No event emission for silent disconnect
        Ok(())
    }

    /// Disconnect from an MCP server
    pub async fn disconnect_server(&self, server_id: &str) -> Result<(), MCPClientError> {
        eprintln!("DEBUG: Plugin disconnect_server called for {}", server_id);

        // Remove and stop the process
        {
            let mut processes = self.processes.lock()
                .map_err(|e| MCPClientError::system_error(&format!("Failed to lock processes: {}", e)))?;
            
            if let Some(mut process) = processes.remove(server_id) {
                process.stop();
                eprintln!("DEBUG: Plugin stopped process for server {}", server_id);
            }
        }

        // Remove connection info
        {
            let mut connections = self.connections.lock()
                .map_err(|e| MCPClientError::system_error(&format!("Failed to lock connections: {}", e)))?;
            connections.remove(server_id);
        }

        // Emit disconnection event
        let event = ConnectionEvent {
            server_id: server_id.to_string(),
            status: "disconnected".to_string(),
            reason: Some("User requested disconnection".to_string()),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            command: None,
            args: None,
        };
        self.emit_connection_event(event);

        Ok(())
    }

    /// Check if a server is connected through the plugin
    pub fn is_server_connected(&self, server_id: &str) -> Result<bool, String> {
        let connections = self.connections.lock()
            .map_err(|e| format!("Failed to lock connections: {}", e))?;
        
        Ok(connections.contains_key(server_id))
    }

    /// List tools from an MCP server through the plugin
    pub async fn list_tools(&self, server_id: &str) -> Result<serde_json::Value, MCPClientError> {
        eprintln!("DEBUG: Plugin list_tools called for server {}", server_id);

        let mut processes = self.processes.lock()
            .map_err(|e| MCPClientError::system_error(&format!("Failed to lock processes: {}", e)))?;

        if let Some(process) = processes.get_mut(server_id) {
            // Check if the process is still running using the public method
            match process.check_process_status() {
                Ok(true) => {
                    eprintln!(
                        "DEBUG: Plugin MCP process for server {} is still running",
                        server_id
                    );
                }
                Ok(false) => {
                    eprintln!(
                        "DEBUG: Plugin MCP process for server {} has exited",
                        server_id
                    );

                    // Emit process exit event for list_tools
                    let event = ConnectionEvent {
                        server_id: server_id.to_string(),
                        status: "disconnected".to_string(),
                        reason: Some("Process exited during tool listing".to_string()),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        command: None,
                        args: None,
                    };
                    self.emit_connection_event(event);

                    return Err(MCPClientError::new(
                        crate::error::ErrorCategory::Connection,
                        "PROCESS_EXITED",
                        &format!("MCP process for server {} has exited", server_id),
                    )
                    .with_suggestions(vec![
                        "Check server logs for errors",
                        "Verify server configuration is correct",
                        "Try reconnecting to the server",
                    ]));
                }
                Err(e) => {
                    eprintln!(
                        "DEBUG: Plugin error checking process status for server {}: {}",
                        server_id, e
                    );
                    return Err(MCPClientError::new(
                        crate::error::ErrorCategory::System,
                        "STATUS_CHECK_FAILED",
                        "Error checking MCP process status",
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        "Try reconnecting to the server",
                        "Restart the application if the issue persists",
                    ]));
                }
            }

            // Create the tools/list JSON-RPC message
            let message_id = process.next_message_id();
            let list_tools_message = serde_json::json!({
                "jsonrpc": "2.0",
                "id": message_id,
                "method": "tools/list",
                "params": {}
            });

            // Send the message
            if let Err(e) = process.send_message_sync(list_tools_message) {
                return Err(e);
            }

            // Read the response with 5 second timeout
            match process.read_response(message_id as u64, 5000) {
                Ok(response) => {
                    eprintln!(
                        "DEBUG: Plugin got tools response for server {}: {}",
                        server_id, response
                    );

                    // Extract the result from the JSON-RPC response
                    if let Some(result) = response.get("result") {
                        Ok(result.clone())
                    } else if let Some(error) = response.get("error") {
                        Err(MCPClientError::protocol_error(&format!(
                            "MCP server returned error: {}",
                            error
                        )))
                    } else {
                        Err(MCPClientError::protocol_error(
                            "Invalid JSON-RPC response: missing result and error",
                        ))
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            return Err(MCPClientError::new(
                crate::error::ErrorCategory::Connection,
                "NO_PROCESS",
                &format!("No active MCP process found for server {}", server_id),
            )
            .with_suggestions(vec![
                "Ensure the server is connected",
                "Try connecting to the server first",
                "Check that the server ID is correct",
            ]));
        }
    }

    /// Execute a tool on an MCP server through the plugin
    pub async fn execute_tool(&self, server_id: &str, tool_name: &str, arguments: serde_json::Value) -> Result<(serde_json::Value, u64), MCPClientError> {
        eprintln!("DEBUG: Plugin execute_tool called for server {} tool {} with args: {}", server_id, tool_name, arguments);

        let start_time = std::time::Instant::now();
        let mut processes = self.processes.lock()
            .map_err(|e| MCPClientError::system_error(&format!("Failed to lock processes: {}", e)))?;

        if let Some(process) = processes.get_mut(server_id) {
            // Check if the process is still running using the public method
            match process.check_process_status() {
                Ok(true) => {
                    eprintln!(
                        "DEBUG: Plugin MCP process for server {} is still running",
                        server_id
                    );
                }
                Ok(false) => {
                    eprintln!(
                        "DEBUG: Plugin MCP process for server {} has exited",
                        server_id
                    );

                    // Emit process exit event for execute_tool
                    let event = ConnectionEvent {
                        server_id: server_id.to_string(),
                        status: "disconnected".to_string(),
                        reason: Some("Process exited during tool execution".to_string()),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        command: None,
                        args: None,
                    };
                    self.emit_connection_event(event);

                    return Err(MCPClientError::new(
                        crate::error::ErrorCategory::Connection,
                        "PROCESS_EXITED",
                        &format!("MCP process for server {} has exited", server_id),
                    )
                    .with_suggestions(vec![
                        "Check server logs for errors",
                        "Verify server configuration is correct",
                        "Try reconnecting to the server",
                    ]));
                }
                Err(e) => {
                    eprintln!(
                        "DEBUG: Plugin error checking process status for server {}: {}",
                        server_id, e
                    );
                    return Err(MCPClientError::new(
                        crate::error::ErrorCategory::System,
                        "STATUS_CHECK_FAILED",
                        "Error checking MCP process status",
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        "Try reconnecting to the server",
                        "Restart the application if the issue persists",
                    ]));
                }
            }

            // Create the tools/call JSON-RPC message
            let message_id = process.next_message_id();
            let call_tool_message = serde_json::json!({
                "jsonrpc": "2.0",
                "id": message_id,
                "method": "tools/call",
                "params": {
                    "name": tool_name,
                    "arguments": arguments
                }
            });

            eprintln!("DEBUG: Plugin sending tool call message: {}", call_tool_message);

            // Send the message
            if let Err(e) = process.send_message_sync(call_tool_message) {
                return Err(e);
            }

            // Read the response with 10 second timeout for tool execution
            match process.read_response(message_id as u64, 10000) {
                Ok(response) => {
                    let duration_ms = start_time.elapsed().as_millis() as u64;
                    eprintln!(
                        "DEBUG: Plugin got tool response for server {} in {}ms: {}",
                        server_id, duration_ms, response
                    );

                    // Extract the result from the JSON-RPC response
                    if let Some(result) = response.get("result") {
                        Ok((result.clone(), duration_ms))
                    } else if let Some(error) = response.get("error") {
                        Err(MCPClientError::new(
                            crate::error::ErrorCategory::Protocol,
                            "TOOL_EXECUTION_ERROR",
                            &format!("Tool '{}' execution failed", tool_name),
                        )
                        .with_details(&format!("MCP server returned error: {}", error))
                        .with_suggestions(vec![
                            "Check the tool parameters are correct",
                            "Verify the tool exists on this server",
                            "Review server logs for more details",
                        ]))
                    } else {
                        Err(MCPClientError::protocol_error(
                            "Invalid JSON-RPC response: missing result and error",
                        ))
                    }
                }
                Err(e) => {
                    return Err(e);
                }
            }
        } else {
            return Err(MCPClientError::new(
                crate::error::ErrorCategory::Connection,
                "NO_PROCESS",
                &format!("No active MCP process found for server {}", server_id),
            )
            .with_suggestions(vec![
                "Ensure the server is connected",
                "Try connecting to the server first",
                "Check that the server ID is correct",
            ]));
        }
    }
}

impl<R: Runtime> Default for ConnectionRegistry<R> {
    fn default() -> Self {
        Self::new()
    }
}