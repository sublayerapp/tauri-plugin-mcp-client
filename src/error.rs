/// Enhanced error handling with specific error types, codes, and categories
use serde::{Deserialize, Serialize};
use std::fmt;

/// Error categories for classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ErrorCategory {
    Connection,
    Permission,
    Timeout,
    Protocol,
    Command,
    Configuration,
    Database,
    System,
}

impl fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ErrorCategory::Connection => write!(f, "CONNECTION"),
            ErrorCategory::Permission => write!(f, "PERMISSION"),
            ErrorCategory::Timeout => write!(f, "TIMEOUT"),
            ErrorCategory::Protocol => write!(f, "PROTOCOL"),
            ErrorCategory::Command => write!(f, "COMMAND"),
            ErrorCategory::Configuration => write!(f, "CONFIGURATION"),
            ErrorCategory::Database => write!(f, "DATABASE"),
            ErrorCategory::System => write!(f, "SYSTEM"),
        }
    }
}

/// Enhanced error type with category, code, message, and context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocollieError {
    pub category: ErrorCategory,
    pub code: String,
    pub message: String,
    pub details: Option<String>,
    pub suggestions: Vec<String>,
}

impl ProtocollieError {
    pub fn new(category: ErrorCategory, code: &str, message: &str) -> Self {
        Self {
            category,
            code: code.to_string(),
            message: message.to_string(),
            details: None,
            suggestions: Vec::new(),
        }
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_suggestion(mut self, suggestion: &str) -> Self {
        self.suggestions.push(suggestion.to_string());
        self
    }

    pub fn with_suggestions(mut self, suggestions: Vec<&str>) -> Self {
        self.suggestions
            .extend(suggestions.into_iter().map(|s| s.to_string()));
        self
    }

    /// Create a command not found error
    pub fn command_not_found(command: &str) -> Self {
        Self::new(
            ErrorCategory::Command,
            "CMD_NOT_FOUND",
            &format!("Command '{}' not found", command),
        )
        .with_details(&format!(
            "The command '{}' is not installed or not in your PATH",
            command
        ))
        .with_suggestions(vec![
            &format!("Install Node.js if using '{}' or 'npx' commands", command),
            "Check that the command is installed and accessible",
            "Verify your PATH environment variable includes the command location",
        ])
    }

    /// Create a permission denied error
    pub fn permission_denied(resource: &str) -> Self {
        Self::new(
            ErrorCategory::Permission,
            "PERMISSION_DENIED",
            &format!("Permission denied accessing {}", resource),
        )
        .with_details(&format!("You don't have permission to access {}", resource))
        .with_suggestions(vec![
            "Check file permissions for the resource",
            "Run with appropriate user permissions",
            "Verify you have execute permissions for the command",
        ])
    }

    /// Create a connection timeout error
    pub fn connection_timeout(target: &str, timeout_ms: u64) -> Self {
        Self::new(
            ErrorCategory::Timeout,
            "CONNECTION_TIMEOUT",
            &format!("Connection to {} timed out after {}ms", target, timeout_ms),
        )
        .with_details(&format!(
            "The server did not respond within {}ms",
            timeout_ms
        ))
        .with_suggestions(vec![
            "Check if the server is running",
            "Verify network connectivity",
            "Try increasing the timeout value",
        ])
    }

    /// Create a protocol error
    pub fn protocol_error(details: &str) -> Self {
        Self::new(
            ErrorCategory::Protocol,
            "PROTOCOL_ERROR",
            "Invalid MCP protocol response",
        )
        .with_details(details)
        .with_suggestions(vec![
            "Verify the server implements MCP protocol correctly",
            "Check server logs for protocol errors",
            "Ensure server and client protocol versions are compatible",
        ])
    }

    /// Create a configuration error
    pub fn configuration_error(field: &str, details: &str) -> Self {
        Self::new(
            ErrorCategory::Configuration,
            "CONFIG_ERROR",
            &format!("Invalid configuration for {}", field),
        )
        .with_details(details)
        .with_suggestions(vec![
            "Check the configuration format",
            "Verify all required fields are provided",
            "Review configuration examples in documentation",
        ])
    }

    /// Create a database error
    pub fn database_error(operation: &str, details: &str) -> Self {
        Self::new(
            ErrorCategory::Database,
            "DATABASE_ERROR",
            &format!("Database {} failed", operation),
        )
        .with_details(details)
        .with_suggestions(vec![
            "Restart the application to reinitialize database",
            "Check available disk space",
            "Verify database file permissions",
        ])
    }

    /// Create a system error
    pub fn system_error(details: &str) -> Self {
        Self::new(
            ErrorCategory::System,
            "SYSTEM_ERROR",
            "System operation failed",
        )
        .with_details(details)
        .with_suggestions(vec![
            "Check system resources (memory, disk space)",
            "Verify system permissions",
            "Try restarting the application",
        ])
    }
}

impl fmt::Display for ProtocollieError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "[{}:{}] {}", self.category, self.code, self.message)?;
        if let Some(details) = &self.details {
            write!(f, "\nDetails: {}", details)?;
        }
        if !self.suggestions.is_empty() {
            write!(f, "\nSuggestions:")?;
            for suggestion in &self.suggestions {
                write!(f, "\n• {}", suggestion)?;
            }
        }
        Ok(())
    }
}

/// Analyze a generic error string and convert to structured error
pub fn analyze_error(error_str: &str) -> ProtocollieError {
    let error_lower = error_str.to_lowercase();

    // Command not found errors
    if error_lower.contains("no such file or directory")
        || error_lower.contains("command not found")
    {
        let command = extract_command_from_error(error_str).unwrap_or("unknown");
        return ProtocollieError::command_not_found(command);
    }

    // Permission errors
    if error_lower.contains("permission denied") {
        let resource = extract_resource_from_error(error_str).unwrap_or("resource");
        return ProtocollieError::permission_denied(resource);
    }

    // Timeout errors
    if error_lower.contains("timeout") {
        return ProtocollieError::connection_timeout("server", 5000);
    }

    // Protocol errors
    if error_lower.contains("invalid json")
        || error_lower.contains("protocol")
        || error_lower.contains("json-rpc")
    {
        return ProtocollieError::protocol_error(error_str);
    }

    // Database errors
    if error_lower.contains("database") || error_lower.contains("sqlite") {
        return ProtocollieError::database_error("operation", error_str);
    }

    // Configuration errors
    if error_lower.contains("config")
        || error_lower.contains("missing")
        || error_lower.contains("invalid")
    {
        return ProtocollieError::configuration_error("field", error_str);
    }

    // Default to system error
    ProtocollieError::system_error(error_str)
}

/// Extract command name from error message
fn extract_command_from_error(error: &str) -> Option<&str> {
    if let Some(start) = error.find("'") {
        if let Some(end) = error[start + 1..].find("'") {
            return Some(&error[start + 1..start + 1 + end]);
        }
    }
    None
}

/// Extract resource name from error message  
fn extract_resource_from_error(error: &str) -> Option<&str> {
    if let Some(start) = error.find("'") {
        if let Some(end) = error[start + 1..].find("'") {
            return Some(&error[start + 1..start + 1 + end]);
        }
    }
    None
}

// Legacy compatibility functions for gradual migration
pub fn format_database_error(error: &str) -> String {
    analyze_error(error).to_string()
}

pub fn format_connection_error(server_name: &str, error: &str) -> String {
    let mut analyzed = analyze_error(error);
    // Customize for server context
    if analyzed.category == ErrorCategory::Command {
        analyzed.message = format!(
            "Failed to connect to '{}': {}",
            server_name, analyzed.message
        );
    }
    analyzed.to_string()
}

pub fn format_tool_execution_error(tool_name: &str, error: &str) -> String {
    let mut analyzed = analyze_error(error);
    // Customize for tool context
    analyzed.message = format!("Tool '{}': {}", tool_name, analyzed.message);
    analyzed.to_string()
}

pub fn format_config_error(error: &str) -> String {
    analyze_error(error).to_string()
}

pub fn format_app_data_error() -> String {
    ProtocollieError::system_error("Unable to access application data directory").to_string()
}