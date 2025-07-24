# Troubleshooting Guide

This guide helps you diagnose and fix common issues when using the `tauri-plugin-mcp` plugin.

## Quick Diagnosis

### Plugin Health Check

First, verify the plugin is working:

```typescript
import { mcp } from 'tauri-plugin-mcp-api';

try {
  const health = await mcp.healthCheck();
  console.log('Plugin status:', health);
} catch (error) {
  console.error('Plugin not available:', error);
}
```

Expected healthy response:
```json
{
  "status": "healthy",
  "version": "0.1.0",
  "plugin_name": "tauri-plugin-mcp",
  "initialized": true
}
```

## Common Issues

### 1. Plugin Not Found

**Error:**
```
Error: Plugin not found: tauri-plugin-mcp
```

**Cause:** Plugin not properly registered in Tauri configuration.

**Solutions:**

#### Check Plugin Registration
Ensure the plugin is registered in `src-tauri/src/lib.rs`:

```rust
use tauri_plugin_mcp;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_mcp::init()) // ✅ Must be here
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

#### Check Dependencies
Verify `Cargo.toml` includes the plugin dependency:

```toml
[dependencies]
tauri-plugin-mcp = "0.1.0"
```

#### Clean and Rebuild
```bash
# Clean build artifacts
cd src-tauri
cargo clean

# Rebuild
cd ..
npm run tauri build
```

### 2. Command Not Found

**Error:**
```
Error: Command not found: health_check
Error: Command not found: plugin_connect_server
```

**Cause:** Plugin commands not registered in the main application.

**Solutions:**

#### Verify Command Registration
Some Tauri setups require explicit command registration. Check if your `lib.rs` has:

```rust
.plugin(tauri_plugin_mcp::init())
.invoke_handler(tauri::generate_handler![
    // Your other commands here
    // Plugin commands are auto-registered by the plugin
])
```

#### Check Plugin Order
Ensure plugin registration comes before invoke_handler:

```rust
// ✅ Correct order
.plugin(tauri_plugin_mcp::init())
.invoke_handler(tauri::generate_handler![/* commands */])

// ❌ Wrong order
.invoke_handler(tauri::generate_handler![/* commands */])
.plugin(tauri_plugin_mcp::init())
```

### 3. Server Connection Failures

**Error:**
```
Error: Failed to connect to server: Process failed to start
Error: Server process exited with code 1
```

**Cause:** MCP server executable issues or invalid command/arguments.

**Solutions:**

#### Verify Server Command
Test the MCP server command manually:

```bash
# Test if the command works directly
node path/to/your/mcp-server.js

# Or whatever your server command is
python mcp-server.py
go run server.go
```

#### Check PATH
Ensure the MCP server command is in your system PATH:

```bash
# Check if command exists
which node
which python
which your-mcp-server-command
```

#### Validate Server Arguments
```typescript
// ✅ Correct format
await mcp.connectServer({
  server_id: 'my-server',
  command: 'node',
  args: ['path/to/server.js', '--port', '3000']
});

// ❌ Common mistakes
await mcp.connectServer({
  server_id: 'my-server',
  command: 'node path/to/server.js', // ❌ Don't combine command and args
  args: []
});
```

#### Check Server Protocol Compliance
Ensure your MCP server implements the protocol correctly:

```javascript
// Your server should respond to these methods:
// - initialize
// - tools/list  
// - tools/call

// And return proper JSON-RPC 2.0 responses
{
  "jsonrpc": "2.0",
  "id": 1,
  "result": { /* response data */ }
}
```

### 4. Message ID Correlation Errors

**Error:**
```
DEBUG: Got response for different ID: 1 (expected 2)
Error: Response timeout waiting for ID 2
```

**Cause:** MCP server not using correct message IDs in responses.

**Solutions:**

#### Check Server ID Handling
Your MCP server must return the same ID it received:

```javascript
// ✅ Correct ID handling
function handleMessage(message) {
  const { id, method, params } = message;
  
  return {
    jsonrpc: '2.0',
    id: id, // ✅ Use the same ID from request
    result: { /* response */ }
  };
}

// ❌ Wrong ID handling
function handleMessage(message) {
  return {
    jsonrpc: '2.0',
    id: 1, // ❌ Hardcoded ID
    result: { /* response */ }
  };
}
```

#### Enable Debug Logging
Add logging to see message flow:

```bash
# Set debug environment variable
RUST_LOG=debug npm run tauri dev
```

### 5. TypeScript Import Errors

**Error:**
```
Module not found: Can't resolve 'tauri-plugin-mcp-api'
Type 'ConnectionInfo' not found
```

**Cause:** TypeScript API package not installed or not properly imported.

**Solutions:**

#### Install TypeScript Package
```bash
npm install tauri-plugin-mcp-api
# or
pnpm add tauri-plugin-mcp-api  
# or
yarn add tauri-plugin-mcp-api
```

#### Check Import Syntax
```typescript
// ✅ Correct imports
import { mcp, onServerConnected } from 'tauri-plugin-mcp-api';
import type { ConnectionInfo, ExecuteToolRequest } from 'tauri-plugin-mcp-api';

// ❌ Wrong imports
import mcp from 'tauri-plugin-mcp-api'; // ❌ No default export
import { ConnectionInfo } from 'tauri-plugin-mcp-api'; // ❌ Should be type import
```

#### Verify Package Installation
```bash
# Check if package is installed
npm list tauri-plugin-mcp-api

# Check package contents
ls node_modules/tauri-plugin-mcp-api/
```

### 6. Event Listener Issues

**Error:**
```
Error: Event listener not receiving events
Events firing but callback not called
```

**Cause:** Event listener registration issues or timing problems.

**Solutions:**

#### Check Event Registration
```typescript
// ✅ Correct event listener setup
import { onServerConnected } from 'tauri-plugin-mcp-api';

useEffect(() => {
  const setupListener = async () => {
    const unlisten = await onServerConnected((event) => {
      console.log('Server connected:', event.server_id);
    });
    
    return unlisten;
  };
  
  let unlisten: (() => void) | undefined;
  setupListener().then(fn => unlisten = fn);
  
  return () => {
    if (unlisten) unlisten();
  };
}, []);
```

#### Verify Event Names
```typescript
// ✅ Use exported constants
import { EVENT_SERVER_CONNECTED } from 'tauri-plugin-mcp-api';

// ❌ Don't use hardcoded strings
await listen('mcp://server-connected', callback); // Might be wrong
```

### 7. Tool Execution Failures

**Error:**
```
Error: Tool 'my-tool' not found
Error: Tool execution timeout
Error: Invalid tool arguments
```

**Cause:** Tool doesn't exist, server issues, or parameter problems.

**Solutions:**

#### Verify Tool Exists
```typescript
// List tools first to verify availability
const tools = await mcp.listTools('server-id');
console.log('Available tools:', tools.tools.map(t => t.name));

// Check if your tool is in the list
const hasMyTool = tools.tools.some(t => t.name === 'my-tool');
if (!hasMyTool) {
  console.error('Tool not available on server');
}
```

#### Validate Arguments
```typescript
// Check tool schema before executing
const tools = await mcp.listTools('server-id');
const myTool = tools.tools.find(t => t.name === 'my-tool');

if (myTool) {
  console.log('Tool schema:', myTool.inputSchema);
  
  // Ensure your arguments match the schema
  await mcp.executeTool({
    server_id: 'server-id',
    tool_name: 'my-tool',
    arguments: {
      // Match the required properties from inputSchema
      param1: 'value1',
      param2: 42
    }
  });
}
```

### 8. Performance Issues

**Error:**
```
Tool execution taking too long
Multiple connection timeouts
High memory usage
```

**Cause:** Server performance issues or resource constraints.

**Solutions:**

#### Monitor Resource Usage
```typescript
// Track execution times
const start = Date.now();
const result = await mcp.executeTool(request);
const duration = Date.now() - start;
console.log(`Tool executed in ${duration}ms`);

// The plugin also provides duration_ms in the response
console.log(`Server reported: ${result.duration_ms}ms`);
```

#### Optimize Server Configuration
```bash
# Increase Node.js memory limit for MCP servers
node --max-old-space-size=4096 your-mcp-server.js

# Use production mode
NODE_ENV=production node your-mcp-server.js
```

#### Connection Pooling
```typescript
// Don't repeatedly connect/disconnect
// Keep connections alive and reuse them
const connections = new Map();

async function getOrCreateConnection(serverId: string, command: string, args: string[]) {
  if (!connections.has(serverId)) {
    await mcp.connectServer({ server_id: serverId, command, args });
    connections.set(serverId, true);
  }
  return serverId;
}
```

## Advanced Debugging

### Enable Verbose Logging

#### Rust Side (Plugin)
```bash
RUST_LOG=tauri_plugin_mcp=debug npm run tauri dev
```

#### TypeScript Side (API)
```typescript
// Add debug logging to your code
import { mcp } from 'tauri-plugin-mcp-api';

const originalExecuteTool = mcp.executeTool;
mcp.executeTool = async (request) => {
  console.log('Executing tool:', request);
  const start = Date.now();
  try {
    const result = await originalExecuteTool(request);
    console.log(`Tool completed in ${Date.now() - start}ms:`, result);
    return result;
  } catch (error) {
    console.error(`Tool failed after ${Date.now() - start}ms:`, error);
    throw error;
  }
};
```

### Test with Minimal Example

Create a minimal test case:

```typescript
// minimal-test.js
import { mcp } from 'tauri-plugin-mcp-api';

async function test() {
  try {
    // 1. Health check
    console.log('1. Testing health check...');
    const health = await mcp.healthCheck();
    console.log('✅ Health check passed:', health);

    // 2. Simple connection (using the example echo server)
    console.log('2. Testing connection...');
    await mcp.connectServer({
      server_id: 'test',
      command: 'node',
      args: ['examples/basic-integration/echo-server.js']
    });
    console.log('✅ Connection successful');

    // 3. List tools
    console.log('3. Testing tool listing...');
    const tools = await mcp.listTools('test');
    console.log('✅ Tools listed:', tools.tools.length);

    // 4. Execute tool
    console.log('4. Testing tool execution...');
    const result = await mcp.executeTool({
      server_id: 'test',
      tool_name: 'echo',
      arguments: { message: 'test' }
    });
    console.log('✅ Tool executed:', result.result);

    // 5. Disconnect
    console.log('5. Testing disconnection...');
    await mcp.disconnectServer('test');
    console.log('✅ Disconnection successful');

    console.log('🎉 All tests passed!');
  } catch (error) {
    console.error('❌ Test failed:', error);
  }
}

test();
```

### Network and Process Debugging

#### Check Process Status
```bash
# On macOS/Linux, check if your MCP server process is running
ps aux | grep your-mcp-server

# On Windows  
tasklist | findstr your-mcp-server
```

#### Monitor Network Activity
```bash
# Check if MCP server is listening on expected ports (if applicable)
netstat -an | grep :3000

# Or use lsof on macOS/Linux
lsof -i :3000
```

### Check File Permissions

```bash
# Ensure MCP server script is executable
chmod +x your-mcp-server.js

# Check file exists and is readable
ls -la your-mcp-server.js
```

## Getting Help

If you're still experiencing issues:

### 1. Collect Debug Information

Create a debug report:

```typescript
// debug-report.js
import { mcp } from 'tauri-plugin-mcp-api';

async function generateDebugReport() {
  const report = {
    timestamp: new Date().toISOString(),
    plugin: null,
    connections: null,
    system: {
      platform: navigator.platform,
      userAgent: navigator.userAgent
    },
    error: null
  };

  try {
    report.plugin = await mcp.healthCheck();
  } catch (error) {
    report.error = error.message;
  }

  try {
    report.connections = await mcp.listConnections();
  } catch (error) {
    report.error = error.message;
  }

  console.log('Debug Report:', JSON.stringify(report, null, 2));
  return report;
}
```

### 2. Search Existing Issues

Check the plugin repository for similar issues:
- Look for error messages in issue titles
- Check closed issues for solutions
- Review troubleshooting discussions

### 3. Create a Minimal Reproduction

When reporting issues, provide:

1. **Minimal code example** that reproduces the problem
2. **Complete error messages** including stack traces
3. **Environment information** (OS, Node.js version, Tauri version)
4. **MCP server details** (command, arguments, implementation)
5. **Debug logs** with `RUST_LOG=debug`

### 4. Community Resources

- Plugin documentation: [README.md](README.md)
- Migration guide: [MIGRATION_GUIDE.md](MIGRATION_GUIDE.md)
- Example implementations: [examples/](examples/)
- Testing guide: [TESTING.md](TESTING.md)

## Preventive Measures

### Development Best Practices

1. **Always check plugin health** before using other functions
2. **Validate MCP servers manually** before integrating
3. **Use structured error handling** with try-catch blocks
4. **Monitor connection states** with event listeners
5. **Test with the example echo server** first
6. **Keep debug logging** during development
7. **Use TypeScript** for better error catching
8. **Write integration tests** for your specific use case

### Production Checklist

- [ ] Plugin health check passes
- [ ] All MCP servers tested independently
- [ ] Error handling covers all failure modes
- [ ] Event listeners properly managed
- [ ] Performance tested under load
- [ ] Resource cleanup implemented
- [ ] Logging configured appropriately
- [ ] Backup plans for server failures

Following these practices will help you avoid most common issues and quickly resolve any that do occur.