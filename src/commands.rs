use serde::{Deserialize, Serialize};
use tauri::{command, AppHandle, Runtime, State, Window};
use crate::registry::{ConnectionRegistry, ConnectionInfo};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthCheckResponse {
    pub status: String,
    pub version: String,
    pub plugin_name: String,
    pub initialized: bool,
}

/// Health check command to verify plugin communication works
#[command]
pub async fn health_check<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
) -> Result<HealthCheckResponse, String> {
    println!("Plugin health_check command called!");
    Ok(HealthCheckResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        plugin_name: "tauri-plugin-mcp-client".to_string(),
        initialized: true,
    })
}

/// Get connection statuses from the plugin registry
#[command]
pub async fn get_connection_statuses<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    registry: State<'_, ConnectionRegistry>,
) -> Result<Vec<ConnectionInfo>, String> {
    println!("Plugin get_connection_statuses command called!");
    registry.get_connection_statuses()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConnectServerRequest {
    pub server_id: String,
    pub command: String,
    pub args: Vec<String>,
}

/// Connect to an MCP server through the plugin (parallel to main system)
#[command]
pub async fn plugin_connect_server<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    registry: State<'_, ConnectionRegistry>,
    request: ConnectServerRequest,
) -> Result<String, String> {
    println!("Plugin connect_server command called for server: {}", request.server_id);
    
    match registry.connect_server(request.server_id.clone(), request.command, request.args).await {
        Ok(()) => {
            println!("Plugin successfully connected to server: {}", request.server_id);
            Ok(format!("Successfully connected to server: {}", request.server_id))
        }
        Err(e) => {
            println!("Plugin failed to connect to server {}: {}", request.server_id, e);
            Err(format!("Failed to connect: {}", e))
        }
    }
}

/// Disconnect from an MCP server through the plugin
#[command]
pub async fn plugin_disconnect_server<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    registry: State<'_, ConnectionRegistry>,
    server_id: String,
) -> Result<String, String> {
    println!("Plugin disconnect_server command called for server: {}", server_id);
    
    match registry.disconnect_server(&server_id).await {
        Ok(()) => {
            println!("Plugin successfully disconnected from server: {}", server_id);
            Ok(format!("Successfully disconnected from server: {}", server_id))
        }
        Err(e) => {
            println!("Plugin failed to disconnect from server {}: {}", server_id, e);
            Err(format!("Failed to disconnect: {}", e))
        }
    }
}

/// List tools from an MCP server through the plugin
#[command]
pub async fn plugin_list_tools<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    registry: State<'_, ConnectionRegistry>,
    server_id: String,
) -> Result<serde_json::Value, String> {
    println!("Plugin list_tools command called for server: {}", server_id);
    
    match registry.list_tools(&server_id).await {
        Ok(tools) => {
            println!("Plugin successfully listed tools for server: {}", server_id);
            Ok(tools)
        }
        Err(e) => {
            println!("Plugin failed to list tools for server {}: {}", server_id, e);
            Err(format!("Failed to list tools: {}", e))
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteToolRequest {
    pub server_id: String,
    pub tool_name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExecuteToolResponse {
    pub result: serde_json::Value,
    pub duration_ms: u64,
}

/// Execute a tool on an MCP server through the plugin
#[command]
pub async fn plugin_execute_tool<R: Runtime>(
    _app: AppHandle<R>,
    _window: Window<R>,
    registry: State<'_, ConnectionRegistry>,
    request: ExecuteToolRequest,
) -> Result<ExecuteToolResponse, String> {
    println!("Plugin execute_tool command called for server: {} tool: {}", request.server_id, request.tool_name);
    
    match registry.execute_tool(&request.server_id, &request.tool_name, request.arguments).await {
        Ok((result, duration_ms)) => {
            println!("Plugin successfully executed tool {} for server: {} in {}ms", request.tool_name, request.server_id, duration_ms);
            Ok(ExecuteToolResponse {
                result,
                duration_ms,
            })
        }
        Err(e) => {
            println!("Plugin failed to execute tool {} for server {}: {}", request.tool_name, request.server_id, e);
            Err(format!("Failed to execute tool: {}", e))
        }
    }
}