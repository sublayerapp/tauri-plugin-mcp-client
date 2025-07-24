/// Tauri MCP Plugin Test Suite
/// 
/// This module contains comprehensive tests for the Tauri MCP plugin,
/// including unit tests and integration tests.

// Note: mock_mcp_server tests are included in mock_mcp_server.rs file
// Other tests are in separate files and run automatically

#[cfg(test)]
mod test_runner {
    /// Test that basic plugin structures are available
    #[test]
    fn test_plugin_imports() {
        // Test that we can import the main plugin structures
        use tauri_plugin_mcp_client::{
            registry::ConnectionRegistry,
            process::MCPProcess,
            error::MCPClientError,
        };
        
        // Create basic structures to verify they work
        let _process = MCPProcess::new("test".to_string());
        let _registry: ConnectionRegistry<tauri::Wry> = ConnectionRegistry::new();
        
        // Test should pass if compilation succeeds
        assert!(true);
    }
    
    /// Test error categorization
    #[test]
    fn test_error_categories() {
        use tauri_plugin_mcp_client::error::{MCPClientError, ErrorCategory};
        
        let error = MCPClientError::new(
            ErrorCategory::Connection,
            "TEST_ERROR",
            "Test error message"
        );
        
        assert_eq!(error.code, "TEST_ERROR");
        assert!(error.message.contains("Test error"));
    }
}