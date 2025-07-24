# Tauri MCP Plugin

A comprehensive Tauri plugin for integrating Model Context Protocol (MCP) servers into desktop applications. This plugin provides a robust, production-ready solution for managing MCP server connections, executing tools, and handling real-time events.

## Features

- 🚀 **MCP Stdio transport Support** - Complete JSON-RPC 2.0 implementation
- 🔗 **Multi-Server Management** - Connect to multiple MCP servers simultaneously  
- 🛠️ **Tool Execution** - Execute MCP tools with full parameter support
- 📡 **Real-time Events** - Connection status updates and process monitoring
- 🧵 **Thread-Safe** - Atomic message ID generation and concurrent operations
- 🎯 **TypeScript Support** - Fully typed API with comprehensive interfaces
- 📚 **Well Documented** - Complete API reference and integration examples

## Quick Start

### Installation

Add the plugin to your Tauri application directly from GitHub:

#### Option 1: Git Dependencies (Recommended)

**Add to your `src-tauri/Cargo.toml`:**
```toml
[dependencies]
tauri-plugin-mcp-client = { git = "https://github.com/username/tauri-plugin-mcp-client" }
```

**Install TypeScript API:**
```bash
# Install from GitHub
npm install github:username/tauri-plugin-mcp-client#subdirectory=guest-js
# or with pnpm
pnpm add github:username/tauri-plugin-mcp-client#subdirectory=guest-js
```

#### Option 2: Local Development

**Clone the repository:**
```bash
git clone https://github.com/username/tauri-plugin-mcp-client
cd tauri-plugin-mcp-client
```

**Add to your `src-tauri/Cargo.toml`:**
```toml
[dependencies]
tauri-plugin-mcp-client = { path = "../tauri-plugin-mcp-client" }
```

**Build and link TypeScript API:**
```bash
cd tauri-plugin-mcp-client/guest-js
pnpm install
pnpm build
```

Then in your frontend `package.json`:
```json
{
  "dependencies": {
    "tauri-plugin-mcp-client-api": "file:../tauri-plugin-mcp-client/guest-js"
  }
}

### Basic Setup

#### 1. Register the Plugin (Rust)

In your `src-tauri/src/lib.rs`:

```rust
use tauri_plugin_mcp_client;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_mcp_client::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### 2. Use the API (TypeScript)

```typescript
import { mcp, onServerConnected } from 'tauri-plugin-mcp-client-api';

// Connect to an MCP server
await mcp.connectServer({
  server_id: 'my-server',
  command: 'node',
  args: ['path/to/mcp-server.js']
});

// List available tools
const tools = await mcp.listTools('my-server');
console.log('Available tools:', tools);

// Execute a tool
const result = await mcp.executeTool({
  server_id: 'my-server',
  tool_name: 'echo',
  arguments: { message: 'Hello, World!' }
});

// Listen for connection events
onServerConnected((event) => {
  console.log(`Server ${event.server_id} connected!`);
});
```

## API Reference

### Core Functions

#### `mcp.healthCheck()`
Check plugin health and initialization status.

```typescript
const health = await mcp.healthCheck();
console.log(health.status); // "healthy"
```

#### `mcp.connectServer(request)`
Connect to an MCP server.

```typescript
await mcp.connectServer({
  server_id: 'unique-server-id',
  command: 'node',
  args: ['server.js', '--port', '3000']
});
```

#### `mcp.listConnections()`
Get status of all connected servers.

```typescript
const connections = await mcp.listConnections();
connections.forEach(conn => {
  console.log(`${conn.server_id}: ${conn.status}`);
});
```

#### `mcp.listTools(serverId)`
List available tools from a connected server.

```typescript
const tools = await mcp.listTools('my-server');
tools.tools.forEach(tool => {
  console.log(`${tool.name}: ${tool.description}`);
});
```

#### `mcp.executeTool(request)`
Execute a tool on a connected server.

```typescript
const result = await mcp.executeTool({
  server_id: 'my-server',
  tool_name: 'file_search',
  arguments: {
    query: 'README',
    path: '/home/user/projects'
  }
});
```

#### `mcp.disconnectServer(serverId)`
Disconnect from an MCP server.

```typescript
await mcp.disconnectServer('my-server');
```

### Event Listeners

The plugin provides real-time events for connection monitoring:

#### `onServerConnected(callback)`
Listen for server connection events.

```typescript
const unlisten = await onServerConnected((event) => {
  console.log(`Connected to ${event.server_id}`);
});

// Stop listening
unlisten();
```

#### `onServerDisconnected(callback)`
Listen for server disconnection events.

```typescript
await onServerDisconnected((event) => {
  console.log(`Disconnected from ${event.server_id}: ${event.reason}`);
});
```

#### `onConnectionChanged(callback)`
Listen for any connection status changes.

```typescript
await onConnectionChanged((event) => {
  console.log(`${event.server_id} status: ${event.status}`);
});
```

#### `onProcessError(callback)`
Listen for MCP process errors.

```typescript
await onProcessError((event) => {
  console.error(`Process error for ${event.server_id}:`, event.reason);
});
```

## TypeScript Interfaces

### Core Types

```typescript
interface ConnectServerRequest {
  server_id: string;
  command: string;
  args: string[];
}

interface ConnectionInfo {
  server_id: string;
  command: string;
  args: string[];
  status: string;
  connected_at?: number;
}

interface ExecuteToolRequest {
  server_id: string;
  tool_name: string;
  arguments: any;
}

interface ExecuteToolResponse {
  result: any;
  duration_ms: number;
}

interface Tool {
  name: string;
  description: string;
  inputSchema: {
    type: 'object';
    properties: Record<string, ToolParameter>;
    required?: string[];
  };
}
```

### Event Types

```typescript
interface ConnectionEvent {
  server_id: string;
  status: string;
  reason?: string;
  timestamp: number;
  command?: string;
  args?: string[];
}
```

## Error Handling

The plugin provides comprehensive error handling with detailed error messages:

```typescript
try {
  await mcp.connectServer({
    server_id: 'test',
    command: 'nonexistent-command',
    args: []
  });
} catch (error) {
  console.error('Connection failed:', error);
  // Error includes category, code, message, and suggestions
}
```

### Error Categories

- **Connection** - Server connection and communication errors
- **Protocol** - JSON-RPC and MCP protocol errors
- **System** - Process management and system-level errors
- **Configuration** - Invalid parameters or configuration

## Advanced Usage

### Multiple Server Management

```typescript
const servers = [
  { id: 'server1', command: 'node', args: ['server1.js'] },
  { id: 'server2', command: 'python', args: ['server2.py'] },
  { id: 'server3', command: 'go', args: ['run', 'server3.go'] }
];

// Connect to all servers
await Promise.all(
  servers.map(server => 
    mcp.connectServer({
      server_id: server.id,
      command: server.command,
      args: server.args
    })
  )
);

// Execute tools on different servers
const results = await Promise.all([
  mcp.executeTool({ server_id: 'server1', tool_name: 'search', arguments: { query: 'test' } }),
  mcp.executeTool({ server_id: 'server2', tool_name: 'analyze', arguments: { data: [1,2,3] } }),
  mcp.executeTool({ server_id: 'server3', tool_name: 'process', arguments: { input: 'hello' } })
]);
```

### Connection State Management

```typescript
class MCPManager {
  private connections = new Map<string, boolean>();

  async initialize() {
    // Listen for all connection events
    await onConnectionChanged((event) => {
      this.connections.set(event.server_id, event.status === 'connected');
      this.onConnectionChange(event);
    });
  }

  async connectIfNeeded(serverId: string, command: string, args: string[]) {
    if (!this.connections.get(serverId)) {
      await mcp.connectServer({ server_id: serverId, command, args });
    }
  }

  async executeToolSafely(serverId: string, toolName: string, args: any) {
    if (!this.connections.get(serverId)) {
      throw new Error(`Server ${serverId} not connected`);
    }
    
    return await mcp.executeTool({
      server_id: serverId,
      tool_name: toolName,
      arguments: args
    });
  }

  private onConnectionChange(event: ConnectionEvent) {
    console.log(`Connection ${event.server_id}: ${event.status}`);
  }
}
```

## Development

### Building from Source

```bash
# Clone the repository
git clone <repository-url>
cd tauri-plugin-mcp

# Build the Rust plugin
cargo build

# Build the TypeScript API
cd guest-js
pnpm install
pnpm build
```

### Running Tests

```bash
# Run Rust tests
cargo test

# Run TypeScript tests
cd guest-js
pnpm test:run
```

See [TESTING.md](TESTING.md) for comprehensive testing documentation.

## Examples

- [Basic Integration](examples/basic-integration/) - Simple MCP server connection
- [Game Integration](examples/game-integration/) - Using MCP in a game context
- [Multi-Server Setup](examples/multi-server/) - Managing multiple MCP servers

## Coming Soon: Registry Distribution

We plan to publish this plugin to official registries for easier installation:

### Future Registry Installation

**Cargo (Rust):**
```bash
cargo add tauri-plugin-mcp-client
```

**npm (TypeScript):**
```bash
npm install tauri-plugin-mcp-client-api
# or
pnpm add tauri-plugin-mcp-client-api
# or
yarn add tauri-plugin-mcp-client-api
```

For now, please use the GitHub-based installation methods described above.

## Migration Guide

Migrating from internal MCP implementations? See [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md) for step-by-step instructions.

## Troubleshooting

### Common Issues

**Plugin not found error**
```
Error: Plugin not found: tauri-plugin-mcp
```
Ensure the plugin is properly registered in your `src-tauri/src/lib.rs` file.

**Command not found error**
```
Error: Command not found: health_check
```
Make sure all plugin commands are registered in both the plugin and your main application.

**Server connection failures**
- Verify the MCP server command and arguments are correct
- Check that the MCP server executable is in your PATH
- Ensure the server supports the MCP protocol version (2024-11-05)

**Message ID correlation errors**
This usually indicates a bug in the MCP server implementation. The plugin uses atomic message ID generation to prevent conflicts.

For more troubleshooting information, see the [Troubleshooting Guide](TROUBLESHOOTING.md).

## Contributing

1. Fork the repository
2. Create a feature branch: `git checkout -b feature/amazing-feature`
3. Make your changes and add tests
4. Ensure all tests pass: `cargo test && cd guest-js && pnpm test:run`
5. Commit your changes: `git commit -m 'Add amazing feature'`
6. Push to the branch: `git push origin feature/amazing-feature`
7. Open a Pull Request

## License

This project is licensed under the MIT OR Apache-2.0 license.

## Changelog

### v0.1.0
- Initial release
- MCP over stdio support
- Multi-server management
- Real-time events
- TypeScript API
