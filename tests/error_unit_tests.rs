use tauri_plugin_mcp_client::error::*;

#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_error_category_display() {
        assert_eq!(ErrorCategory::Connection.to_string(), "CONNECTION");
        assert_eq!(ErrorCategory::Permission.to_string(), "PERMISSION");
        assert_eq!(ErrorCategory::Timeout.to_string(), "TIMEOUT");
        assert_eq!(ErrorCategory::Protocol.to_string(), "PROTOCOL");
        assert_eq!(ErrorCategory::Command.to_string(), "COMMAND");
        assert_eq!(ErrorCategory::Configuration.to_string(), "CONFIGURATION");
        assert_eq!(ErrorCategory::Database.to_string(), "DATABASE");
        assert_eq!(ErrorCategory::System.to_string(), "SYSTEM");
    }

    #[test]
    fn test_mcp_client_error_new() {
        let error = MCPClientError::new(ErrorCategory::Command, "TEST_CODE", "Test message");
        assert_eq!(error.category, ErrorCategory::Command);
        assert_eq!(error.code, "TEST_CODE");
        assert_eq!(error.message, "Test message");
        assert!(error.details.is_none());
        assert!(error.suggestions.is_empty());
    }

    #[test]
    fn test_mcp_client_error_with_details() {
        let error = MCPClientError::new(ErrorCategory::Command, "TEST_CODE", "Test message")
            .with_details("Additional details");
        assert_eq!(error.details, Some("Additional details".to_string()));
    }

    #[test]
    fn test_mcp_client_error_with_suggestion() {
        let error = MCPClientError::new(ErrorCategory::Command, "TEST_CODE", "Test message")
            .with_suggestion("Try this fix");
        assert_eq!(error.suggestions, vec!["Try this fix".to_string()]);
    }

    #[test]
    fn test_mcp_client_error_with_suggestions() {
        let error = MCPClientError::new(ErrorCategory::Command, "TEST_CODE", "Test message")
            .with_suggestions(vec!["Fix 1", "Fix 2", "Fix 3"]);
        assert_eq!(error.suggestions, vec!["Fix 1", "Fix 2", "Fix 3"]);
    }

    #[test]
    fn test_command_not_found_error() {
        let error = MCPClientError::command_not_found("node");
        assert_eq!(error.category, ErrorCategory::Command);
        assert_eq!(error.code, "CMD_NOT_FOUND");
        assert!(error.message.contains("node"));
        assert!(error.details.is_some());
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_permission_denied_error() {
        let error = MCPClientError::permission_denied("/etc/passwd");
        assert_eq!(error.category, ErrorCategory::Permission);
        assert_eq!(error.code, "PERMISSION_DENIED");
        assert!(error.message.contains("/etc/passwd"));
        assert!(error.details.is_some());
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_connection_timeout_error() {
        let error = MCPClientError::connection_timeout("localhost:8080", 5000);
        assert_eq!(error.category, ErrorCategory::Timeout);
        assert_eq!(error.code, "CONNECTION_TIMEOUT");
        assert!(error.message.contains("localhost:8080"));
        assert!(error.message.contains("5000ms"));
        assert!(error.details.is_some());
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_protocol_error() {
        let error = MCPClientError::protocol_error("Invalid JSON received");
        assert_eq!(error.category, ErrorCategory::Protocol);
        assert_eq!(error.code, "PROTOCOL_ERROR");
        assert_eq!(error.message, "Invalid MCP protocol response");
        assert_eq!(error.details, Some("Invalid JSON received".to_string()));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_configuration_error() {
        let error = MCPClientError::configuration_error("timeout", "Value must be positive");
        assert_eq!(error.category, ErrorCategory::Configuration);
        assert_eq!(error.code, "CONFIG_ERROR");
        assert!(error.message.contains("timeout"));
        assert_eq!(error.details, Some("Value must be positive".to_string()));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_database_error() {
        let error = MCPClientError::database_error("query", "Table not found");
        assert_eq!(error.category, ErrorCategory::Database);
        assert_eq!(error.code, "DATABASE_ERROR");
        assert!(error.message.contains("query"));
        assert_eq!(error.details, Some("Table not found".to_string()));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_system_error() {
        let error = MCPClientError::system_error("Out of memory");
        assert_eq!(error.category, ErrorCategory::System);
        assert_eq!(error.code, "SYSTEM_ERROR");
        assert_eq!(error.message, "System operation failed");
        assert_eq!(error.details, Some("Out of memory".to_string()));
        assert!(!error.suggestions.is_empty());
    }

    #[test]
    fn test_error_display() {
        let error = MCPClientError::new(ErrorCategory::Command, "TEST_CODE", "Test message")
            .with_details("Additional details")
            .with_suggestion("Try this fix");
        
        let display_str = error.to_string();
        assert!(display_str.contains("[COMMAND:TEST_CODE]"));
        assert!(display_str.contains("Test message"));
        assert!(display_str.contains("Details: Additional details"));
        assert!(display_str.contains("Suggestions:"));
        assert!(display_str.contains("• Try this fix"));
    }

    #[test]
    fn test_analyze_error_command_not_found() {
        let error = analyze_error("bash: 'nonexistent': command not found");
        assert_eq!(error.category, ErrorCategory::Command);
        assert_eq!(error.code, "CMD_NOT_FOUND");
    }

    #[test]
    fn test_analyze_error_no_such_file() {
        let error = analyze_error("sh: no such file or directory");
        assert_eq!(error.category, ErrorCategory::Command);
        assert_eq!(error.code, "CMD_NOT_FOUND");
    }

    #[test]
    fn test_analyze_error_permission_denied() {
        let error = analyze_error("permission denied accessing '/root/secret'");
        assert_eq!(error.category, ErrorCategory::Permission);
        assert_eq!(error.code, "PERMISSION_DENIED");
    }

    #[test]
    fn test_analyze_error_timeout() {
        let error = analyze_error("connection timeout occurred");
        assert_eq!(error.category, ErrorCategory::Timeout);
        assert_eq!(error.code, "CONNECTION_TIMEOUT");
    }

    #[test]
    fn test_analyze_error_protocol() {
        let error = analyze_error("invalid json received from server");
        assert_eq!(error.category, ErrorCategory::Protocol);
        assert_eq!(error.code, "PROTOCOL_ERROR");
    }

    #[test] 
    fn test_analyze_error_database() {
        let error = analyze_error("database connection failed");
        assert_eq!(error.category, ErrorCategory::Database);
        assert_eq!(error.code, "DATABASE_ERROR");
    }

    #[test]
    fn test_analyze_error_configuration() {
        let error = analyze_error("missing required config field");
        assert_eq!(error.category, ErrorCategory::Configuration);
        assert_eq!(error.code, "CONFIG_ERROR");
    }

    #[test]
    fn test_analyze_error_system_default() {
        let error = analyze_error("unexpected system failure");
        assert_eq!(error.category, ErrorCategory::System);
        assert_eq!(error.code, "SYSTEM_ERROR");
    }

    #[test]
    fn test_format_database_error() {
        let result = format_database_error("connection failed");
        assert!(result.contains("SYSTEM_ERROR"));
    }

    #[test]
    fn test_format_connection_error() {
        let result = format_connection_error("test-server", "command not found");
        assert!(result.contains("Failed to connect to 'test-server'"));
    }

    #[test]
    fn test_format_tool_execution_error() {
        let result = format_tool_execution_error("test-tool", "execution failed");
        assert!(result.contains("Tool 'test-tool'"));
    }

    #[test]
    fn test_format_config_error() {
        let result = format_config_error("invalid configuration");
        assert!(result.contains("CONFIG_ERROR"));
    }

    #[test]
    fn test_format_app_data_error() {
        let result = format_app_data_error();
        assert!(result.contains("SYSTEM_ERROR"));
        assert!(result.contains("application data directory"));
    }
}