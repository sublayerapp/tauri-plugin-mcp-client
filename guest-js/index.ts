/**
 * TypeScript API for tauri-plugin-mcp-client
 * 
 * This module provides a TypeScript interface for interacting with MCP servers
 * through the Tauri MCP plugin.
 */

import { invoke } from '@tauri-apps/api/core';
import { listen, UnlistenFn } from '@tauri-apps/api/event';

// Health check response type
export interface HealthCheckResponse {
  status: string;
  version: string;
  plugin_name: string;
  initialized: boolean;
}

// Basic types - will be expanded in future stories
export interface ConnectionStatus {
  connected: boolean;
  server_id: string;
  status: string;
}

export interface ConnectionInfo {
  server_id: string;
  command: string;
  args: string[];
  status: string;
  connected_at?: number; // Unix timestamp
}

export interface ConnectServerRequest {
  server_id: string;
  command: string;
  args: string[];
}

// Tool-related types (matching Protocollie's interfaces)
export interface ToolParameter {
  type: string;
  description?: string;
  enum?: string[];
  items?: any;
  properties?: Record<string, any>;
}

export interface Tool {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: Record<string, ToolParameter>;
    required?: string[];
  };
}

export interface ToolsResponse {
  tools: Tool[];
}

// Tool execution types (matching Protocollie's interfaces)
export interface ExecuteToolRequest {
  server_id: string;
  tool_name: string;
  arguments: any;
}

export interface ExecuteToolResponse {
  result: any;
  duration_ms: number;
}

// Tool execution result (for history/tracking)
export interface ToolExecutionResult {
  tool_name: string;
  result: any;
  duration_ms: number;
  timestamp: number;
  error?: string;
  arguments?: any;
}

// Event system types
export const EVENT_CONNECTION_CHANGED = 'mcp://connection-changed';
export const EVENT_SERVER_CONNECTED = 'mcp://server-connected';
export const EVENT_SERVER_DISCONNECTED = 'mcp://server-disconnected';
export const EVENT_PROCESS_ERROR = 'mcp://process-error';

export interface ConnectionEvent {
  server_id: string;
  status: string; // "connected", "disconnected", "error"
  reason?: string;
  timestamp: number;
  command?: string;
  args?: string[];
}

export interface MCPClient {
  // Plugin functionality will be added in future stories
  healthCheck(): Promise<HealthCheckResponse>;
  getConnectionStatus(id: string): Promise<ConnectionStatus>;
  listConnections(): Promise<ConnectionInfo[]>;
  connectServer(request: ConnectServerRequest): Promise<string>;
  disconnectServer(serverId: string): Promise<string>;
  listTools(serverId: string): Promise<any>; // Raw JSON-RPC response for now
  executeTool(request: ExecuteToolRequest): Promise<ExecuteToolResponse>;
}

// Health check - now actually calls the plugin
export async function healthCheck(): Promise<HealthCheckResponse> {
  console.log('Attempting health_check command...');
  return await invoke('health_check');
}

// Placeholder functions for future implementation
export async function getConnectionStatus(id: string): Promise<ConnectionStatus> {
  return {
    connected: false,
    server_id: id,
    status: "not_implemented"
  };
}

export async function listConnections(): Promise<ConnectionInfo[]> {
  console.log('Attempting get_connection_statuses command...');
  return await invoke('get_connection_statuses');
}

// Connect to an MCP server through the plugin (parallel system)
export async function connectServer(request: ConnectServerRequest): Promise<string> {
  console.log('Attempting plugin_connect_server command for:', request.server_id);
  return await invoke('plugin_connect_server', { request });
}

// Disconnect from an MCP server through the plugin
export async function disconnectServer(serverId: string): Promise<string> {
  console.log('Attempting plugin_disconnect_server command for:', serverId);
  return await invoke('plugin_disconnect_server', { serverId });
}

// List tools from an MCP server through the plugin
export async function listTools(serverId: string): Promise<any> {
  console.log('Attempting plugin_list_tools command for:', serverId);
  return await invoke('plugin_list_tools', { serverId });
}

// Execute a tool on an MCP server through the plugin
export async function executeTool(request: ExecuteToolRequest): Promise<ExecuteToolResponse> {
  console.log('Attempting plugin_execute_tool command for server:', request.server_id, 'tool:', request.tool_name);
  return await invoke('plugin_execute_tool', { request });
}

// Event listener helpers
export async function onConnectionChanged(callback: (event: ConnectionEvent) => void): Promise<UnlistenFn> {
  return await listen<ConnectionEvent>(EVENT_CONNECTION_CHANGED, (event) => {
    console.log('MCP connection changed:', event.payload);
    callback(event.payload);
  });
}

export async function onServerConnected(callback: (event: ConnectionEvent) => void): Promise<UnlistenFn> {
  return await listen<ConnectionEvent>(EVENT_SERVER_CONNECTED, (event) => {
    console.log('MCP server connected:', event.payload);
    callback(event.payload);
  });
}

export async function onServerDisconnected(callback: (event: ConnectionEvent) => void): Promise<UnlistenFn> {
  return await listen<ConnectionEvent>(EVENT_SERVER_DISCONNECTED, (event) => {
    console.log('MCP server disconnected:', event.payload);
    callback(event.payload);
  });
}

export async function onProcessError(callback: (event: ConnectionEvent) => void): Promise<UnlistenFn> {
  return await listen<ConnectionEvent>(EVENT_PROCESS_ERROR, (event) => {
    console.log('MCP process error:', event.payload);
    callback(event.payload);
  });
}

// Convenience function to listen to all MCP events
export async function onAllMCPEvents(callback: (event: ConnectionEvent) => void): Promise<UnlistenFn[]> {
  const unlisteners = await Promise.all([
    onConnectionChanged(callback),
    onServerConnected(callback),
    onServerDisconnected(callback),
    onProcessError(callback),
  ]);
  return unlisteners;
}

// Export client implementation
export const mcp: MCPClient = {
  healthCheck,
  getConnectionStatus,
  listConnections,
  connectServer,
  disconnectServer,
  listTools,
  executeTool
};