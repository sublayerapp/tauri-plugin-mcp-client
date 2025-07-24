#!/usr/bin/env node

/**
 * Simple Echo MCP Server
 * 
 * This is a basic MCP server implementation that demonstrates:
 * - MCP protocol compliance (JSON-RPC 2.0)
 * - Tool registration and execution
 * - Parameter validation
 * - Response formatting
 * 
 * Usage: node echo-server.js
 */

const readline = require('readline');

// Create readline interface for stdin/stdout communication
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

// Server information
const SERVER_INFO = {
  name: 'echo-server-example',
  version: '1.0.0'
};

// Available tools
const TOOLS = [
  {
    name: 'echo',
    description: 'Echo back the input message with optional formatting',
    inputSchema: {
      type: 'object',
      properties: {
        message: {
          type: 'string',
          description: 'Message to echo back'
        },
        prefix: {
          type: 'string',
          description: 'Optional prefix to add to the message',
          default: 'Echo: '
        },
        uppercase: {
          type: 'boolean',
          description: 'Whether to convert the message to uppercase',
          default: false
        }
      },
      required: ['message']
    }
  },
  {
    name: 'reverse',
    description: 'Reverse the input string',
    inputSchema: {
      type: 'object',
      properties: {
        text: {
          type: 'string',
          description: 'Text to reverse'
        }
      },
      required: ['text']
    }
  },
  {
    name: 'count_words',
    description: 'Count the number of words in a text',
    inputSchema: {
      type: 'object',
      properties: {
        text: {
          type: 'string',
          description: 'Text to count words in'
        }
      },
      required: ['text']
    }
  }
];

// Listen for incoming messages
rl.on('line', (line) => {
  try {
    const message = JSON.parse(line);
    const response = handleMessage(message);
    if (response) {
      console.log(JSON.stringify(response));
    }
  } catch (error) {
    // Ignore malformed JSON - MCP servers should be resilient
    console.error('Error parsing message:', error.message);
  }
});

// Handle graceful shutdown
process.on('SIGINT', () => {
  console.error('Echo server shutting down...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  console.error('Echo server shutting down...');
  process.exit(0);
});

/**
 * Handle incoming JSON-RPC messages
 * @param {Object} message - The JSON-RPC message
 * @returns {Object|null} - The response object or null if no response needed
 */
function handleMessage(message) {
  const { jsonrpc, method, id, params } = message;

  // Validate JSON-RPC version
  if (jsonrpc !== '2.0') {
    return createErrorResponse(id, -32600, 'Invalid Request', 'JSON-RPC version must be 2.0');
  }

  switch (method) {
    case 'initialize':
      return handleInitialize(id, params);
    
    case 'tools/list':
      return handleListTools(id);
    
    case 'tools/call':
      return handleToolCall(id, params);
    
    default:
      return createErrorResponse(id, -32601, 'Method not found', `Unknown method: ${method}`);
  }
}

/**
 * Handle initialize request
 */
function handleInitialize(id, params) {
  const { protocolVersion, capabilities } = params || {};

  // Validate protocol version
  if (protocolVersion !== '2024-11-05') {
    return createErrorResponse(id, -32602, 'Invalid params', 
      `Unsupported protocol version: ${protocolVersion}. Expected: 2024-11-05`);
  }

  return {
    jsonrpc: '2.0',
    id,
    result: {
      protocolVersion: '2024-11-05',
      capabilities: {
        tools: {}
      },
      serverInfo: SERVER_INFO
    }
  };
}

/**
 * Handle tools/list request
 */
function handleListTools(id) {
  return {
    jsonrpc: '2.0',
    id,
    result: {
      tools: TOOLS
    }
  };
}

/**
 * Handle tools/call request
 */
function handleToolCall(id, params) {
  if (!params || !params.name) {
    return createErrorResponse(id, -32602, 'Invalid params', 'Tool name is required');
  }

  const { name, arguments: args } = params;
  const tool = TOOLS.find(t => t.name === name);

  if (!tool) {
    return createErrorResponse(id, -32601, 'Method not found', `Tool not found: ${name}`);
  }

  try {
    const result = executeTools(name, args || {});
    return {
      jsonrpc: '2.0',
      id,
      result
    };
  } catch (error) {
    return createErrorResponse(id, -32603, 'Internal error', error.message);
  }
}

/**
 * Execute a tool by name
 */
function executeTools(toolName, args) {
  switch (toolName) {
    case 'echo':
      return executeEcho(args);
    
    case 'reverse':
      return executeReverse(args);
    
    case 'count_words':
      return executeCountWords(args);
    
    default:
      throw new Error(`Unknown tool: ${toolName}`);
  }
}

/**
 * Execute the echo tool
 */
function executeEcho(args) {
  const { message, prefix = 'Echo: ', uppercase = false } = args;

  if (typeof message !== 'string') {
    throw new Error('Message must be a string');
  }

  let result = prefix + message;
  if (uppercase) {
    result = result.toUpperCase();
  }

  return {
    content: [
      {
        type: 'text',
        text: result
      }
    ]
  };
}

/**
 * Execute the reverse tool
 */
function executeReverse(args) {
  const { text } = args;

  if (typeof text !== 'string') {
    throw new Error('Text must be a string');
  }

  const reversed = text.split('').reverse().join('');

  return {
    content: [
      {
        type: 'text',
        text: reversed
      }
    ]
  };
}

/**
 * Execute the count_words tool
 */
function executeCountWords(args) {
  const { text } = args;

  if (typeof text !== 'string') {
    throw new Error('Text must be a string');
  }

  // Simple word counting - split by whitespace and filter empty strings
  const words = text.trim().split(/\s+/).filter(word => word.length > 0);
  const count = words.length;

  return {
    content: [
      {
        type: 'text',
        text: `Word count: ${count}`
      }
    ]
  };
}

/**
 * Create an error response
 */
function createErrorResponse(id, code, message, data) {
  return {
    jsonrpc: '2.0',
    id,
    error: {
      code,
      message,
      ...(data && { data })
    }
  };
}

// Log server startup
console.error(`Echo MCP Server v${SERVER_INFO.version} starting...`);
console.error(`Available tools: ${TOOLS.map(t => t.name).join(', ')}`);
console.error('Server ready for connections.');