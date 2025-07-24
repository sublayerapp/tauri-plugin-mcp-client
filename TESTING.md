# Tauri MCP Plugin Testing Guide

This document describes how to run and maintain the comprehensive test suite for the Tauri MCP plugin.

## Test Structure Overview

The plugin has a comprehensive test suite covering multiple aspects:

### 1. Rust Tests
- **Unit Tests** (`tests/unit_tests.rs`) - Test individual components and data structures
- **Integration Tests** (`tests/integration_tests.rs`) - Test plugin functionality end-to-end
- **Mock Server Tests** (`tests/mock_mcp_server.rs`) - Test the mock MCP server implementation

### 2. TypeScript Tests
- **Type Tests** (`guest-js/tests/types.test.ts`) - Validate TypeScript interfaces and type safety

## Running Tests

### Prerequisites

1. **Rust Development Environment**
   - Rust 1.70+ with Cargo
   - Required system dependencies (varies by platform)

2. **Node.js Development Environment**
   - Node.js 18+ 
   - pnpm package manager

### Rust Tests

#### Run All Rust Tests
```bash
cd /path/to/tauri-plugin-mcp
cargo test
```

#### Run Specific Test Categories
```bash
# Unit tests only
cargo test --test unit_tests

# Integration tests only
cargo test --test integration_tests

# Mock server tests only
cargo test --test mock_mcp_server

# Library tests only
cargo test --lib
```

#### Run Tests with Output
```bash
# Show test output (useful for debugging)
cargo test -- --nocapture

# Show test names as they run
cargo test -- --nocapture --test-threads=1
```

### TypeScript Tests

#### Setup TypeScript Test Environment
```bash
cd guest-js
pnpm install
```

#### Run TypeScript Tests
```bash
# Run all TypeScript tests
pnpm test:run

# Run specific test file
pnpm test:run tests/types.test.ts

# Run tests in watch mode (for development)
pnpm test
```

## Test Coverage

### Rust Test Coverage

#### Unit Tests (`tests/unit_tests.rs`)
- ✅ Health check response structure validation
- ✅ Connection request structure validation
- ✅ Execute tool request structure validation
- ✅ Error categorization and message formatting
- ✅ Connection info data structure validation
- ✅ JSON-RPC response parsing
- ✅ Message ID uniqueness and sequential generation
- ✅ Tool execution response structure validation

#### Integration Tests (`tests/integration_tests.rs`)
- ✅ Plugin connection lifecycle (connect/disconnect)
- ✅ Message ID generation and concurrency
- ✅ Registry connection status tracking
- ✅ Error handling for invalid commands and non-existent servers
- ✅ JSON-RPC protocol format validation
- ✅ Process cleanup and resource management

#### Mock Server Tests (`tests/mock_mcp_server.rs`)
- ✅ Mock server initialization and response handling
- ✅ Tools listing functionality
- ✅ Tool execution with parameters
- ✅ Error handling for unknown methods
- ✅ JSON-RPC protocol compliance

### TypeScript Test Coverage

#### Type Tests (`guest-js/tests/types.test.ts`)
- ✅ All interface structure validation (15 interfaces)
- ✅ Event constant validation
- ✅ MCPClient interface completeness
- ✅ Optional field handling
- ✅ Type safety verification

## Test Infrastructure

### Mock MCP Server

The plugin includes a comprehensive mock MCP server (`tests/mock_mcp_server.rs`) that:

- Implements the full MCP protocol (initialize, tools/list, tools/call)
- Provides configurable tools for testing
- Supports error simulation
- Can be used as a subprocess for integration testing

**Example Usage:**
```rust
let server = MockMCPServer::new("test-server", "1.0.0")
    .with_echo_tool();

let response = server.handle_message(&json!({
    "jsonrpc": "2.0",
    "id": 1,
    "method": "initialize",
    "params": {"protocolVersion": "2024-11-05", "capabilities": {}}
})).unwrap();
```

### Test Utilities

#### Message ID Testing
The tests verify that the atomic message ID counter works correctly:
- Sequential ID generation (0, 1, 2, 3...)
- Uniqueness across multiple calls
- Thread safety (where applicable)

#### Error Handling Testing
Comprehensive error handling tests cover:
- Invalid commands
- Non-existent servers
- Malformed requests
- Process failures
- Timeout scenarios

## Writing New Tests

### Adding Rust Tests

1. **Unit Tests**: Add to `tests/unit_tests.rs`
   ```rust
   #[test]
   fn test_new_functionality() {
       // Test implementation
       assert_eq!(expected, actual);
   }
   ```

2. **Async Integration Tests**: Add to `tests/integration_tests.rs`
   ```rust
   #[tokio::test]
   async fn test_async_functionality() {
       let registry: ConnectionRegistry<tauri::Wry> = ConnectionRegistry::new();
       // Test async functionality
   }
   ```

3. **Mock Server Tests**: Add to `tests/mock_mcp_server.rs`
   ```rust
   #[test]
   fn test_mock_server_feature() {
       let server = MockMCPServer::new("test", "1.0.0");
       // Test mock server functionality
   }
   ```

### Adding TypeScript Tests

Add to `guest-js/tests/types.test.ts`:
```typescript
it('should validate new interface structure', () => {
  const newInterface: NewInterface = {
    field1: 'value1',
    field2: 123,
  };
  
  expect(newInterface.field1).toBe('value1');
  expect(newInterface.field2).toBe(123);
});
```

## Continuous Integration

### Local Testing Script

Create a script to run all tests:
```bash
#!/bin/bash
set -e

echo "Running Rust tests..."
cargo test

echo "Running TypeScript tests..."
cd guest-js
pnpm test:run
cd ..

echo "All tests passed! ✅"
```

### CI/CD Integration

For GitHub Actions or similar CI systems:
```yaml
name: Test Suite
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
    - uses: actions/checkout@v3
    - uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    - name: Run Rust tests
      run: cargo test
    - uses: actions/setup-node@v3
      with:
        node-version: '18'
    - name: Install pnpm
      run: npm install -g pnpm
    - name: Run TypeScript tests
      run: |
        cd guest-js
        pnpm install
        pnpm test:run
```

## Test Performance

### Current Performance
- **Rust Tests**: ~8 seconds total (including compilation)
- **TypeScript Tests**: ~200ms total
- **Total Test Suite**: <10 seconds

### Optimization Tips
1. Use `cargo test --release` for performance testing
2. Run specific test categories during development
3. Use `--test-threads=1` for debugging race conditions
4. Cache dependencies in CI environments

## Troubleshooting

### Common Issues

#### "Command not found" Errors
- Ensure all plugin commands are registered in both the plugin and main app
- Check that the plugin API package has been rebuilt and reloaded

#### Message ID Correlation Errors
- Verify that hardcoded message IDs match the atomic counter sequence
- Check that message IDs are passed correctly to `read_response()`

#### TypeScript Compilation Errors
- Ensure all types are properly exported from `index.ts`
- Check that optional fields are correctly marked with `?`

#### Test Timeouts
- Increase timeout values for slow operations
- Use simpler test scenarios that don't require real MCP servers

### Debug Mode

Enable debug output during testing:
```bash
# For Rust tests
RUST_LOG=debug cargo test

# For TypeScript tests
DEBUG=1 pnpm test:run
```

## Test Maintenance

### Regular Maintenance Tasks

1. **Update Dependencies**: Keep test dependencies current
2. **Review Test Coverage**: Ensure new features have corresponding tests
3. **Performance Monitoring**: Track test execution time
4. **Documentation**: Keep this guide updated with new test procedures

### Adding New Test Categories

When adding major new functionality:

1. Create dedicated test file (e.g., `tests/new_feature_tests.rs`)
2. Add comprehensive unit tests for components
3. Add integration tests for end-to-end functionality
4. Update this documentation
5. Add CI/CD integration if needed

## Summary

The Tauri MCP plugin has a comprehensive test suite with:
- **28 Rust tests** covering unit, integration, and mock server functionality
- **15 TypeScript tests** covering type safety and interface validation
- **Mock MCP server** for reliable testing
- **Full CI/CD integration** support
- **<10 second** total execution time

This test suite ensures the plugin is reliable, performant, and maintains compatibility across updates.