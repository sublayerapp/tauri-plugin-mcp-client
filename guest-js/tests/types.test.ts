import { describe, it, expect } from 'vitest';
import type {
  HealthCheckResponse,
  ConnectionInfo,
  ConnectServerRequest,
  ExecuteToolRequest,
  ExecuteToolResponse,
  ConnectionEvent,
  Tool,
  ToolsResponse,
  ToolParameter,
  ToolExecutionResult,
  MCPClient
} from '../index';

import {
  EVENT_CONNECTION_CHANGED,
  EVENT_SERVER_CONNECTED,
  EVENT_SERVER_DISCONNECTED,
  EVENT_PROCESS_ERROR,
} from '../index';

describe('TypeScript API Types and Constants', () => {
  describe('Type Structure Validation', () => {
    it('should validate HealthCheckResponse structure', () => {
      const response: HealthCheckResponse = {
        status: 'healthy',
        version: '0.1.0',
        plugin_name: 'tauri-plugin-mcp-client',
        initialized: true,
      };

      expect(response.status).toBe('healthy');
      expect(response.version).toBe('0.1.0');
      expect(response.plugin_name).toBe('tauri-plugin-mcp-client');
      expect(response.initialized).toBe(true);
    });

    it('should validate ConnectionInfo structure', () => {
      const info: ConnectionInfo = {
        server_id: 'test-server',
        command: 'node',
        args: ['server.js'],
        status: 'connected',
        connected_at: 1234567890,
      };

      expect(info.server_id).toBe('test-server');
      expect(info.command).toBe('node');
      expect(info.args).toEqual(['server.js']);
      expect(info.status).toBe('connected');
      expect(info.connected_at).toBe(1234567890);
    });

    it('should validate ConnectServerRequest structure', () => {
      const request: ConnectServerRequest = {
        server_id: 'test-server',
        command: 'node',
        args: ['server.js'],
      };

      expect(request.server_id).toBe('test-server');
      expect(request.command).toBe('node');
      expect(request.args).toEqual(['server.js']);
    });

    it('should validate ExecuteToolRequest structure', () => {
      const request: ExecuteToolRequest = {
        server_id: 'test-server',
        tool_name: 'echo',
        arguments: { message: 'hello world' },
      };

      expect(request.server_id).toBe('test-server');
      expect(request.tool_name).toBe('echo');
      expect(request.arguments).toEqual({ message: 'hello world' });
    });

    it('should validate ExecuteToolResponse structure', () => {
      const response: ExecuteToolResponse = {
        result: {
          content: [{
            type: 'text',
            text: 'Hello, World!'
          }]
        },
        duration_ms: 150,
      };

      expect(response.result).toBeDefined();
      expect(response.duration_ms).toBe(150);
    });

    it('should validate ConnectionEvent structure', () => {
      const event: ConnectionEvent = {
        server_id: 'test-server',
        status: 'connected',
        timestamp: 1234567890,
        reason: 'User requested connection',
        command: 'node',
        args: ['server.js'],
      };

      expect(event.server_id).toBe('test-server');
      expect(event.status).toBe('connected');
      expect(event.timestamp).toBe(1234567890);
      expect(event.reason).toBe('User requested connection');
      expect(event.command).toBe('node');
      expect(event.args).toEqual(['server.js']);
    });

    it('should validate Tool structure', () => {
      const tool: Tool = {
        name: 'echo',
        description: 'Echo tool that returns input',
        inputSchema: {
          type: 'object',
          properties: {
            message: {
              type: 'string',
              description: 'Message to echo'
            }
          },
          required: ['message']
        }
      };

      expect(tool.name).toBe('echo');
      expect(tool.description).toBe('Echo tool that returns input');
      expect(tool.inputSchema.type).toBe('object');
      expect(tool.inputSchema.properties.message.type).toBe('string');
      expect(tool.inputSchema.required).toEqual(['message']);
    });

    it('should validate ToolParameter structure', () => {
      const param: ToolParameter = {
        type: 'string',
        description: 'A string parameter',
        enum: ['option1', 'option2'],
      };

      expect(param.type).toBe('string');
      expect(param.description).toBe('A string parameter');
      expect(param.enum).toEqual(['option1', 'option2']);
    });

    it('should validate ToolsResponse structure', () => {
      const response: ToolsResponse = {
        tools: [
          {
            name: 'echo',
            description: 'Echo tool',
            inputSchema: {
              type: 'object',
              properties: {},
            }
          }
        ]
      };

      expect(response.tools).toHaveLength(1);
      expect(response.tools[0].name).toBe('echo');
    });

    it('should validate ToolExecutionResult structure', () => {
      const result: ToolExecutionResult = {
        tool_name: 'echo',
        result: { content: [{ type: 'text', text: 'Hello' }] },
        duration_ms: 100,
        timestamp: 1234567890,
        arguments: { message: 'Hello' },
      };

      expect(result.tool_name).toBe('echo');
      expect(result.duration_ms).toBe(100);
      expect(result.timestamp).toBe(1234567890);
      expect(result.arguments).toEqual({ message: 'Hello' });
    });
  });

  describe('Event Constants', () => {
    it('should export correct event constants', () => {
      expect(EVENT_CONNECTION_CHANGED).toBe('mcp://connection-changed');
      expect(EVENT_SERVER_CONNECTED).toBe('mcp://server-connected');
      expect(EVENT_SERVER_DISCONNECTED).toBe('mcp://server-disconnected');
      expect(EVENT_PROCESS_ERROR).toBe('mcp://process-error');
    });
  });

  describe('MCPClient Interface', () => {
    it('should define MCPClient interface correctly', () => {
      // This test just validates that the interface compiles
      // We can't test the actual implementation without mocking Tauri
      const mockClient: MCPClient = {
        healthCheck: async () => ({
          status: 'healthy',
          version: '0.1.0',
          plugin_name: 'test',
          initialized: true
        }),
        getConnectionStatus: async () => ({
          connected: true,
          server_id: 'test',
          status: 'connected'
        }),
        listConnections: async () => [],
        connectServer: async () => 'connected',
        disconnectServer: async () => 'disconnected',
        listTools: async () => ({ tools: [] }),
        executeTool: async () => ({
          result: { content: [] },
          duration_ms: 0
        })
      };

      expect(mockClient).toBeDefined();
      expect(typeof mockClient.healthCheck).toBe('function');
      expect(typeof mockClient.getConnectionStatus).toBe('function');
      expect(typeof mockClient.listConnections).toBe('function');
      expect(typeof mockClient.connectServer).toBe('function');
      expect(typeof mockClient.disconnectServer).toBe('function');
      expect(typeof mockClient.listTools).toBe('function');
      expect(typeof mockClient.executeTool).toBe('function');
    });
  });

  describe('Optional Fields', () => {
    it('should handle optional fields in ConnectionInfo', () => {
      const infoWithoutTimestamp: ConnectionInfo = {
        server_id: 'test',
        command: 'node',
        args: [],
        status: 'connected',
        // connected_at is optional
      };

      expect(infoWithoutTimestamp.connected_at).toBeUndefined();
    });

    it('should handle optional fields in ConnectionEvent', () => {
      const minimalEvent: ConnectionEvent = {
        server_id: 'test',
        status: 'connected',
        timestamp: 1234567890,
        // reason, command, args are optional
      };

      expect(minimalEvent.reason).toBeUndefined();
      expect(minimalEvent.command).toBeUndefined();
      expect(minimalEvent.args).toBeUndefined();
    });

    it('should handle optional fields in ToolExecutionResult', () => {
      const minimalResult: ToolExecutionResult = {
        tool_name: 'test',
        result: {},
        duration_ms: 100,
        timestamp: 1234567890,
        // error and arguments are optional
      };

      expect(minimalResult.error).toBeUndefined();
      expect(minimalResult.arguments).toBeUndefined();
    });
  });
});