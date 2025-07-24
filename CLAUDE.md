# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Rust Plugin Development
```bash
# Build the Rust plugin
cargo build

# Run all Rust tests (unit, integration, mock server)
cargo test

# Run specific test categories
cargo test --test unit_tests        # Unit tests only
cargo test --test integration_tests # Integration tests only
cargo test --lib                    # Library tests only

# Run tests with debug output
cargo test -- --nocapture
```

### TypeScript API Development
```bash
# Navigate to TypeScript package
cd guest-js

# Install dependencies
pnpm install

# Build TypeScript API
pnpm build

# Watch mode for development
pnpm dev

# Run TypeScript tests
pnpm test:run

# Watch mode for tests
pnpm test
```

### Full Development Workflow
```bash
# Build both Rust and TypeScript components
cargo build && cd guest-js && pnpm build

# Run complete test suite
cargo test && cd guest-js && pnpm test:run
```

## Architecture Overview

This is a **Tauri plugin** that provides MCP (Model Context Protocol) client capabilities to Tauri applications. The architecture follows a dual-language pattern:

### Core Components

**Rust Backend (`src/`)**:
- `lib.rs` - Plugin initialization and command registration
- `commands.rs` - Tauri command handlers for frontend communication
- `registry.rs` - Connection management and MCP server registry
- `process.rs` - MCP server process lifecycle management
- `error.rs` - Error handling and categorization

**TypeScript Frontend (`guest-js/`)**:
- `index.ts` - Public API exports and type definitions
- Type-safe wrappers around Tauri plugin commands
- Event listener abstractions for real-time updates

### Key Architectural Patterns

**Plugin Structure**: Uses Tauri's plugin system with `Builder::new("mcp")` pattern. The plugin name "mcp" is registered in `lib.rs:24`.

**State Management**: Uses Tauri's `State` management for the `ConnectionRegistry` which tracks MCP server connections across the application lifecycle.

**Async Communication**: All MCP operations are async with proper error propagation from Rust to TypeScript via Tauri's command system.

**Process Management**: MCP servers run as external processes managed through `tokio::process::Command` with stdio communication.

**Event System**: Real-time connection events are emitted from Rust and consumed in TypeScript via Tauri's event system.

## Plugin Integration

To use this plugin in a Tauri application:

1. **Rust side** (`src-tauri/src/lib.rs`):
```rust
use tauri_plugin_mcp_client;

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_mcp_client::init())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

2. **TypeScript side**:
```typescript
import { mcp } from 'tauri-plugin-mcp-client-api';
```

## Testing Strategy

The project uses comprehensive testing across both languages:

- **28 Rust tests**: Unit tests, integration tests, and mock MCP server tests
- **15 TypeScript tests**: Type safety and interface validation
- **Mock MCP Server**: Standalone test server in `tests/mock_mcp_server.rs`
- **Test execution time**: <10 seconds total

## MCP Protocol Implementation

The plugin implements the MCP (Model Context Protocol) specification:
- **Protocol version**: 2024-11-05
- **Transport**: stdio-based JSON-RPC 2.0
- **Core methods**: initialize, tools/list, tools/call
- **Message ID handling**: Atomic counter for unique request correlation

## Build Outputs

- **Rust**: Produces `libtaturi_plugin_mcp_client.rlib` and `.so`/`.dll`/`.dylib`
- **TypeScript**: Produces ESM and CJS builds in `guest-js/dist/` with TypeScript declarations

## Error Handling

The plugin categorizes errors into:
- **Connection**: Server connection and communication errors  
- **Protocol**: JSON-RPC and MCP protocol errors
- **System**: Process management and system-level errors
- **Configuration**: Invalid parameters or configuration

All errors propagate from Rust through Tauri commands to TypeScript with detailed messages and suggested resolutions.