# Basic Integration Example

This example demonstrates the basic usage of the `tauri-plugin-mcp` plugin in a simple Tauri application.

## Features Demonstrated

- Plugin registration and initialization
- Connecting to an MCP server
- Listing available tools
- Executing tools with parameters
- Handling connection events
- Basic error handling

## Project Structure

```
basic-integration/
├── src-tauri/
│   ├── src/
│   │   └── lib.rs          # Plugin registration
│   ├── Cargo.toml          # Rust dependencies
│   └── tauri.conf.json     # Tauri configuration
├── src/
│   ├── App.tsx             # Main React component
│   ├── hooks/
│   │   └── useMCP.ts       # MCP integration hook
│   └── main.tsx            # React entry point
├── package.json            # Frontend dependencies
└── README.md               # This file
```

## Setup Instructions

### 1. Install Dependencies

```bash
# Install frontend dependencies
npm install

# Install Rust dependencies (done automatically by Tauri)
```

### 2. Register the Plugin

The plugin is already registered in `src-tauri/src/lib.rs`:

```rust
use tauri_plugin_mcp;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_mcp::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### 3. Run the Example

```bash
# Development mode
npm run tauri dev

# Production build
npm run tauri build
```

## Usage Example

The main application demonstrates:

### Basic Connection

```typescript
import { mcp } from 'tauri-plugin-mcp-api';

// Connect to an MCP server
await mcp.connectServer({
  server_id: 'echo-server',
  command: 'node',
  args: ['examples/echo-server.js']
});
```

### Tool Discovery and Execution

```typescript
// List available tools
const tools = await mcp.listTools('echo-server');
console.log('Available tools:', tools.tools);

// Execute a tool
const result = await mcp.executeTool({
  server_id: 'echo-server',
  tool_name: 'echo',
  arguments: { message: 'Hello, World!' }
});

console.log('Tool result:', result.result);
```

### Event Handling

```typescript
import { onServerConnected, onServerDisconnected } from 'tauri-plugin-mcp-api';

// Listen for connection events
await onServerConnected((event) => {
  console.log(`Server ${event.server_id} connected at ${new Date(event.timestamp * 1000)}`);
});

await onServerDisconnected((event) => {
  console.log(`Server ${event.server_id} disconnected: ${event.reason}`);
});
```

## Custom Hook Example

The example includes a custom React hook (`useMCP.ts`) that demonstrates best practices:

```typescript
export function useMCP() {
  const [connections, setConnections] = useState<ConnectionInfo[]>([]);
  const [tools, setTools] = useState<Tool[]>([]);
  const [isLoading, setIsLoading] = useState(false);

  const connectToServer = useCallback(async (serverId: string, command: string, args: string[]) => {
    setIsLoading(true);
    try {
      await mcp.connectServer({ server_id: serverId, command, args });
      await refreshConnections();
    } catch (error) {
      console.error('Connection failed:', error);
      throw error;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const executeToolSafe = useCallback(async (serverId: string, toolName: string, args: any) => {
    try {
      return await mcp.executeTool({
        server_id: serverId,
        tool_name: toolName,
        arguments: args
      });
    } catch (error) {
      console.error('Tool execution failed:', error);
      throw error;
    }
  }, []);

  return {
    connections,
    tools,
    isLoading,
    connectToServer,
    executeToolSafe,
    refreshConnections,
    refreshTools
  };
}
```

## Sample MCP Server

The example includes a simple echo server (`echo-server.js`) that demonstrates:

- MCP protocol implementation
- Tool registration
- Parameter handling
- Response formatting

```javascript
#!/usr/bin/env node

const readline = require('readline');

const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout,
  terminal: false
});

rl.on('line', (line) => {
  try {
    const message = JSON.parse(line);
    const response = handleMessage(message);
    if (response) {
      console.log(JSON.stringify(response));
    }
  } catch (e) {
    // Ignore malformed JSON
  }
});

function handleMessage(message) {
  const { method, id, params } = message;

  switch (method) {
    case 'initialize':
      return {
        jsonrpc: '2.0',
        id,
        result: {
          protocolVersion: '2024-11-05',
          capabilities: { tools: {} },
          serverInfo: { name: 'echo-server', version: '1.0.0' }
        }
      };

    case 'tools/list':
      return {
        jsonrpc: '2.0',
        id,
        result: {
          tools: [{
            name: 'echo',
            description: 'Echo back the input message',
            inputSchema: {
              type: 'object',
              properties: {
                message: { type: 'string', description: 'Message to echo' }
              },
              required: ['message']
            }
          }]
        }
      };

    case 'tools/call':
      if (params.name === 'echo') {
        return {
          jsonrpc: '2.0',
          id,
          result: {
            content: [{
              type: 'text',
              text: `Echo: ${params.arguments.message}`
            }]
          }
        };
      }
      break;
  }

  return {
    jsonrpc: '2.0',
    id,
    error: { code: -32601, message: 'Method not found' }
  };
}
```

## Testing

The example includes basic tests to verify functionality:

```bash
# Run tests
npm test
```

## Common Patterns

### Error Handling with User Feedback

```typescript
const connectWithFeedback = async (serverId: string, command: string, args: string[]) => {
  try {
    setStatus('Connecting...');
    await mcp.connectServer({ server_id: serverId, command, args });
    setStatus('Connected successfully!');
  } catch (error) {
    setStatus(`Connection failed: ${error.message}`);
    console.error('Connection error:', error);
  }
};
```

### Tool Validation

```typescript
const executeToolSafely = async (serverId: string, toolName: string, args: any) => {
  // Validate server connection
  const connections = await mcp.listConnections();
  const serverConnection = connections.find(c => c.server_id === serverId);
  
  if (!serverConnection || serverConnection.status !== 'connected') {
    throw new Error(`Server ${serverId} is not connected`);
  }

  // Validate tool exists
  const tools = await mcp.listTools(serverId);
  const tool = tools.tools.find(t => t.name === toolName);
  
  if (!tool) {
    throw new Error(`Tool ${toolName} not found on server ${serverId}`);
  }

  // Execute tool
  return await mcp.executeTool({
    server_id: serverId,
    tool_name: toolName,
    arguments: args
  });
};
```

### Connection State Management

```typescript
const useConnectionState = () => {
  const [connectionStates, setConnectionStates] = useState(new Map());

  useEffect(() => {
    const setupEventListeners = async () => {
      await onConnectionChanged((event) => {
        setConnectionStates(prev => new Map(prev.set(event.server_id, {
          status: event.status,
          timestamp: event.timestamp,
          reason: event.reason
        })));
      });
    };

    setupEventListeners();
  }, []);

  return connectionStates;
};
```

## Next Steps

After understanding this basic example:

1. Try the [Game Integration Example](../game-integration/) for more advanced usage
2. Read the [Migration Guide](../../MIGRATION_GUIDE.md) if migrating existing code
3. Check the [Plugin Documentation](../../README.md) for complete API reference
4. Explore the [Troubleshooting Guide](../../TROUBLESHOOTING.md) for common issues

## Support

If you encounter issues with this example:

1. Verify all dependencies are installed correctly
2. Check that the MCP server is executable and in your PATH
3. Review the browser developer console for error messages
4. Ensure the plugin is properly registered in your Tauri configuration