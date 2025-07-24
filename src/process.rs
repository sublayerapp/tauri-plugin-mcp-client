use crate::error::{ErrorCategory, ProtocollieError};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::process::Child;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
// Removed AppHandle import since we now use system Node.js directly

/// Track pending JSON-RPC requests for debugging and correlation
#[derive(Debug, Clone)]
pub struct PendingRequest {
    pub message_id: u32,
    pub method: String,
    pub timestamp: Instant,
}

/// Check if Node.js is available and provide helpful error message if not
fn check_nodejs_availability() -> Result<String, ProtocollieError> {
    match std::process::Command::new("node").arg("--version").output() {
        Ok(output) => {
            if output.status.success() {
                let version = String::from_utf8_lossy(&output.stdout);
                let version_str = version.trim().to_string();
                eprintln!("DEBUG: Found Node.js version: {}", version_str);
                Ok(version_str)
            } else {
                Err(ProtocollieError::new(
                    ErrorCategory::Command,
                    "NODE_NOT_WORKING",
                    "Node.js is installed but not working properly",
                )
                .with_details("Node.js command returned non-zero exit status")
                .with_suggestions(vec![
                    "Try running 'node --version' in your terminal",
                    "Reinstall Node.js from https://nodejs.org/",
                    "Restart Protocollie after fixing Node.js",
                ]))
            }
        }
        Err(_) => Err(ProtocollieError::new(
            ErrorCategory::Command,
            "NODE_NOT_FOUND",
            "Node.js is required but not found",
        )
        .with_details("Protocollie requires Node.js to run MCP servers")
        .with_suggestions(vec![
            "Download from: https://nodejs.org/",
            "macOS: brew install node",
            "Ubuntu: sudo apt install nodejs npm",
            "Windows: winget install OpenJS.NodeJS",
            "After installing Node.js, restart Protocollie",
        ])),
    }
}

/// Single MCP server process manager
pub struct MCPProcess {
    server_id: String,
    process: Option<Child>,
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<BufReader<std::process::ChildStdout>>,
    stderr_receiver: Option<Receiver<String>>,
    message_counter: AtomicU32,
    pending_requests: Mutex<HashMap<u32, PendingRequest>>,
}

impl MCPProcess {
    pub fn new(server_id: String) -> Self {
        Self {
            server_id,
            process: None,
            stdin: None,
            stdout: None,
            stderr_receiver: None,
            message_counter: AtomicU32::new(0),
            pending_requests: Mutex::new(HashMap::new()),
        }
    }

    /// Generate the next unique message ID for JSON-RPC requests
    pub fn next_message_id(&self) -> u32 {
        self.message_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Track a pending request for debugging and correlation
    pub fn track_request(&self, message_id: u32, method: &str) {
        if let Ok(mut pending) = self.pending_requests.lock() {
            pending.insert(message_id, PendingRequest {
                message_id,
                method: method.to_string(),
                timestamp: Instant::now(),
            });
        }
    }

    /// Remove a completed request from tracking
    pub fn complete_request(&self, message_id: u32) -> Option<PendingRequest> {
        if let Ok(mut pending) = self.pending_requests.lock() {
            pending.remove(&message_id)
        } else {
            None
        }
    }

    /// Test if we can read anything from stdout (diagnostic function)
    pub fn test_stdout_availability(&mut self) -> Result<String, String> {
        eprintln!(
            "DEBUG: Testing stdout availability for server {}",
            self.server_id
        );

        if self.stdout.is_none() {
            return Err("No stdout available".to_string());
        }

        // Check if process is still running
        if let Some(child) = &mut self.process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    return Err(format!("Process has exited with status: {:?}", status));
                }
                Ok(None) => {
                    eprintln!("DEBUG: Process is still running");
                }
                Err(e) => {
                    return Err(format!("Error checking process status: {}", e));
                }
            }
        }

        // Try to read with a very short timeout to see if anything is available
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_millis(100); // Very short timeout
        let stdout = self.stdout.as_mut().unwrap();

        while start_time.elapsed() < timeout {
            let mut line = String::new();
            match stdout.read_line(&mut line) {
                Ok(0) => {
                    return Err("Process closed stdout".to_string());
                }
                Ok(bytes_read) => {
                    return Ok(format!("Read {} bytes: '{}'", bytes_read, line.trim()));
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    // Would block, continue waiting
                    std::thread::sleep(Duration::from_millis(5));
                    continue;
                }
                Err(e) => {
                    return Err(format!("Error reading: {}", e));
                }
            }
        }

        Ok("No data available within timeout".to_string())
    }

    /// Get comprehensive debug information about this process
    pub fn get_debug_info(&mut self) -> serde_json::Value {
        let mut debug_info = serde_json::Map::new();

        // Test basic process health
        if let Some(child) = &mut self.process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    debug_info.insert(
                        "process_status".to_string(),
                        serde_json::json!({
                            "running": false,
                            "exit_status": format!("{:?}", status)
                        }),
                    );
                }
                Ok(None) => {
                    debug_info.insert(
                        "process_status".to_string(),
                        serde_json::json!({
                            "running": true
                        }),
                    );
                }
                Err(e) => {
                    debug_info.insert(
                        "process_status".to_string(),
                        serde_json::json!({
                            "error": format!("{}", e)
                        }),
                    );
                }
            }
        } else {
            debug_info.insert(
                "process_status".to_string(),
                serde_json::json!({
                    "running": false,
                    "error": "No child process available"
                }),
            );
        }

        // Test stdout availability
        match self.test_stdout_availability() {
            Ok(result) => {
                debug_info.insert(
                    "stdout_test".to_string(),
                    serde_json::json!({
                        "success": true,
                        "result": result
                    }),
                );
            }
            Err(e) => {
                debug_info.insert(
                    "stdout_test".to_string(),
                    serde_json::json!({
                        "success": false,
                        "error": e
                    }),
                );
            }
        }

        // Collect any recent stderr
        if let Some(stderr) = self.collect_stderr(500) {
            debug_info.insert("recent_stderr".to_string(), serde_json::json!(stderr));
        } else {
            debug_info.insert(
                "recent_stderr".to_string(),
                serde_json::json!("No stderr available"),
            );
        }

        // Check pipe states
        debug_info.insert(
            "pipe_status".to_string(),
            serde_json::json!({
                "stdin_available": self.stdin.is_some(),
                "stdout_available": self.stdout.is_some(),
                "stderr_receiver_available": self.stderr_receiver.is_some()
            }),
        );

        serde_json::Value::Object(debug_info)
    }

    pub async fn start(&mut self, command: &str, args: &[String]) -> Result<(), ProtocollieError> {
        eprintln!(
            "DEBUG: Starting MCP process for server {} with command: '{}' args: {:?}",
            self.server_id, command, args
        );

        // Check Node.js availability for Node.js-based commands
        if command == "node" || command == "npx" {
            if let Err(nodejs_error) = check_nodejs_availability() {
                return Err(nodejs_error);
            }
        }

        // Spawn MCP server process with stdio pipes for MCP communication
        let mut cmd = std::process::Command::new(command);
        cmd.args(args)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            eprintln!("DEBUG: Failed to spawn MCP server process: {}", e);

            // Create specific error based on command type and error details
            let error_str = e.to_string().to_lowercase();

            if error_str.contains("no such file") || error_str.contains("not found") {
                ProtocollieError::command_not_found(command)
            } else if error_str.contains("permission denied") {
                ProtocollieError::permission_denied(&format!("command '{}'", command))
            } else {
                match command {
                    "node" | "npx" => ProtocollieError::new(
                        ErrorCategory::Command,
                        "NODE_START_FAILED",
                        &format!("Failed to start Node.js MCP server '{}'", command),
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        "Ensure Node.js is installed and in your PATH",
                        "Verify the MCP server script exists and is accessible",
                        "Check you have permission to execute the script",
                    ]),
                    "python" | "python3" => ProtocollieError::new(
                        ErrorCategory::Command,
                        "PYTHON_START_FAILED",
                        &format!("Failed to start Python MCP server '{}'", command),
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        "Ensure Python is installed and in your PATH",
                        "Install required Python packages",
                        "Check you have permission to execute the script",
                    ]),
                    _ => ProtocollieError::new(
                        ErrorCategory::Command,
                        "COMMAND_START_FAILED",
                        &format!("Failed to start MCP server command '{}'", command),
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        &format!("Ensure '{}' is installed and in your PATH", command),
                        "Check you have permission to execute the command",
                        "Verify all required dependencies are installed",
                    ]),
                }
            }
        })?;

        // Capture stderr for debugging and error reporting
        if let Some(stderr) = child.stderr.take() {
            eprintln!("DEBUG: Process has stderr available for capture");
            let (sender, receiver) = channel();
            self.stderr_receiver = Some(receiver);

            let server_id_clone = self.server_id.clone();
            std::thread::spawn(move || {
                use std::io::{BufRead, BufReader};
                let reader = BufReader::new(stderr);
                let mut stderr_lines = Vec::new();

                for line in reader.lines() {
                    match line {
                        Ok(line_content) => {
                            eprintln!("DEBUG: MCP stderr [{}]: {}", server_id_clone, line_content);
                            stderr_lines.push(line_content.clone());

                            // Send individual lines to channel (non-blocking)
                            if sender.send(line_content).is_err() {
                                eprintln!(
                                    "DEBUG: Stderr channel closed for server {}",
                                    server_id_clone
                                );
                                break;
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "DEBUG: Error reading stderr from MCP process {}: {}",
                                server_id_clone, e
                            );
                            break;
                        }
                    }
                }

                // Send accumulated stderr as final message
                if !stderr_lines.is_empty() {
                    let combined_stderr = stderr_lines.join("\n");
                    let _ = sender.send(format!("STDERR_COMPLETE:{}", combined_stderr));
                }

                eprintln!(
                    "DEBUG: Stderr reader thread ended for server {}",
                    server_id_clone
                );
            });
        }

        // Take stdin for writing and stdout for reading
        self.stdin = child.stdin.take();
        if let Some(stdout) = child.stdout.take() {
            eprintln!(
                "DEBUG: Successfully captured stdout for server {}",
                self.server_id
            );
            self.stdout = Some(BufReader::new(stdout));
        } else {
            eprintln!(
                "DEBUG: WARNING - No stdout available for server {}",
                self.server_id
            );
        }

        // Check if stdin is available
        if self.stdin.is_some() {
            eprintln!(
                "DEBUG: Successfully captured stdin for server {}",
                self.server_id
            );
        } else {
            eprintln!(
                "DEBUG: WARNING - No stdin available for server {}",
                self.server_id
            );
        }

        self.process = Some(child);

        eprintln!(
            "DEBUG: MCP process started for server {} - stdin: {}, stdout: {}",
            self.server_id,
            if self.stdin.is_some() {
                "available"
            } else {
                "missing"
            },
            if self.stdout.is_some() {
                "available"
            } else {
                "missing"
            }
        );
        Ok(())
    }

    /// Collect any available stderr output
    pub fn collect_stderr(&mut self, timeout_ms: u64) -> Option<String> {
        if let Some(ref receiver) = self.stderr_receiver {
            let mut stderr_lines = Vec::new();
            let timeout = Duration::from_millis(timeout_ms);
            let start_time = std::time::Instant::now();

            while start_time.elapsed() < timeout {
                match receiver.try_recv() {
                    Ok(line) => {
                        if line.starts_with("STDERR_COMPLETE:") {
                            // Extract the complete stderr
                            let complete_stderr =
                                line.strip_prefix("STDERR_COMPLETE:").unwrap_or("");
                            return Some(complete_stderr.to_string());
                        } else {
                            stderr_lines.push(line);
                        }
                    }
                    Err(std::sync::mpsc::TryRecvError::Empty) => {
                        // No more messages available, wait a bit
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(std::sync::mpsc::TryRecvError::Disconnected) => {
                        // Channel closed, return what we have
                        break;
                    }
                }
            }

            if !stderr_lines.is_empty() {
                Some(stderr_lines.join("\n"))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn send_initialize(&mut self) -> Result<(), ProtocollieError> {
        eprintln!(
            "DEBUG: Starting MCP initialization for server {}",
            self.server_id
        );

        let message_id = self.next_message_id();
        self.track_request(message_id, "initialize");
        let init_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": message_id,
            "method": "initialize",
            "params": {
                "protocolVersion": "2024-11-05",
                "capabilities": {},
                "clientInfo": {
                    "name": "protocollie",
                    "version": "1.0.0"
                }
            }
        });

        eprintln!(
            "DEBUG: Sending initialize message to server {}",
            self.server_id
        );
        self.send_message_sync(init_message)?;
        eprintln!("DEBUG: Initialize message sent successfully");

        // Read the initialize response
        eprintln!("DEBUG: Waiting for initialize response...");
        match self.read_response(message_id as u64, 5000) {
            Ok(response) => {
                eprintln!("DEBUG: Got initialize response: {}", response);
            }
            Err(e) => {
                eprintln!("DEBUG: Failed to read initialize response: {}", e);
                // Collect any stderr that might explain the issue
                if let Some(stderr) = self.collect_stderr(1000) {
                    eprintln!("DEBUG: Stderr during initialize: {}", stderr);
                }
                // Don't fail the connection, some servers might not respond immediately
            }
        }

        // Send initialized notification
        let initialized_notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/initialized"
        });

        eprintln!(
            "DEBUG: Sending initialized notification to server {}",
            self.server_id
        );
        self.send_message_sync(initialized_notification)?;
        eprintln!("DEBUG: Initialized notification sent successfully");

        eprintln!(
            "DEBUG: MCP initialization completed for server {}",
            self.server_id
        );
        Ok(())
    }

    pub fn send_message_sync(
        &mut self,
        message: serde_json::Value,
    ) -> Result<(), ProtocollieError> {
        let stdin = self.stdin.as_mut().ok_or_else(|| {
            ProtocollieError::new(
                ErrorCategory::Connection,
                "NO_STDIN",
                "MCP process not started or stdin not available",
            )
            .with_details("Cannot send message to MCP server without stdin pipe")
            .with_suggestions(vec![
                "Ensure the MCP server process is running",
                "Check that the server was started correctly",
                "Try reconnecting to the server",
            ])
        })?;

        let message_str = serde_json::to_string(&message).map_err(|e| {
            ProtocollieError::new(
                ErrorCategory::Protocol,
                "JSON_SERIALIZE_FAILED",
                "Failed to serialize JSON-RPC message",
            )
            .with_details(&e.to_string())
            .with_suggestions(vec![
                "Check message format is valid JSON",
                "Verify message structure follows JSON-RPC spec",
            ])
        })?;

        eprintln!(
            "DEBUG: Sending to MCP server {}: {}",
            self.server_id, message_str
        );

        writeln!(stdin, "{}", message_str).map_err(|e| {
            ProtocollieError::new(
                ErrorCategory::Connection,
                "WRITE_FAILED",
                "Failed to write message to MCP process",
            )
            .with_details(&e.to_string())
            .with_suggestions(vec![
                "Check if the MCP server process is still running",
                "Verify the process stdin pipe is not broken",
                "Try reconnecting to the server",
            ])
        })?;

        stdin.flush().map_err(|e| {
            ProtocollieError::new(
                ErrorCategory::Connection,
                "FLUSH_FAILED",
                "Failed to flush stdin buffer",
            )
            .with_details(&e.to_string())
            .with_suggestions(vec![
                "Check if the MCP server process is still running",
                "Try reconnecting to the server",
            ])
        })?;

        Ok(())
    }

    pub fn read_response(
        &mut self,
        expected_id: u64,
        timeout_ms: u64,
    ) -> Result<serde_json::Value, ProtocollieError> {
        let stdout = self.stdout.as_mut().ok_or_else(|| {
            ProtocollieError::new(
                ErrorCategory::Connection,
                "NO_STDOUT",
                "MCP process stdout not available",
            )
            .with_details("Cannot read response from MCP server without stdout pipe")
            .with_suggestions(vec![
                "Ensure the MCP server process is running",
                "Check that the server was started correctly",
                "Try reconnecting to the server",
            ])
        })?;

        // Try to read a response with timeout
        let start_time = std::time::Instant::now();
        let timeout = Duration::from_millis(timeout_ms);
        let mut all_output = Vec::new();

        eprintln!(
            "DEBUG: Starting to read response for ID {} with {}ms timeout",
            expected_id, timeout_ms
        );

        while start_time.elapsed() < timeout {
            // Try to read a line (non-blocking would be better, but this is simpler for now)
            let mut line = String::new();
            match stdout.read_line(&mut line) {
                Ok(0) => {
                    eprintln!(
                        "DEBUG: MCP process closed stdout - collected {} lines total",
                        all_output.len()
                    );
                    if !all_output.is_empty() {
                        eprintln!(
                            "DEBUG: All stdout output received before close: {:?}",
                            all_output
                        );
                    }
                    return Err(ProtocollieError::new(
                        ErrorCategory::Connection,
                        "STDOUT_CLOSED",
                        "MCP process closed stdout unexpectedly",
                    )
                    .with_details("The server terminated the connection")
                    .with_suggestions(vec![
                        "Check server logs for errors",
                        "Verify server configuration is correct",
                        "Try reconnecting to the server",
                    ]));
                }
                Ok(bytes_read) => {
                    eprintln!(
                        "DEBUG: Read {} bytes from stdout: '{}'",
                        bytes_read,
                        line.trim()
                    );
                    let line = line.trim();
                    if line.is_empty() {
                        eprintln!("DEBUG: Skipping empty line");
                        continue;
                    }

                    // Store all output for debugging
                    all_output.push(line.to_string());

                    eprintln!(
                        "DEBUG: Received from MCP server {} (line {}): {}",
                        self.server_id,
                        all_output.len(),
                        line
                    );

                    // Try to parse as JSON
                    match serde_json::from_str::<serde_json::Value>(line) {
                        Ok(json) => {
                            eprintln!("DEBUG: Successfully parsed JSON: {}", json);
                            // Check if this is the response we're looking for
                            if let Some(response_id) = json.get("id") {
                                eprintln!("DEBUG: JSON has ID field: {}", response_id);
                                if response_id.as_u64() == Some(expected_id) {
                                    eprintln!(
                                        "DEBUG: Found matching response for ID {}",
                                        expected_id
                                    );
                                    return Ok(json);
                                } else {
                                    eprintln!(
                                        "DEBUG: Got response for different ID: {} (expected {})",
                                        response_id, expected_id
                                    );
                                    continue;
                                }
                            } else {
                                eprintln!(
                                    "DEBUG: Got JSON without ID (probably a notification): {}",
                                    line
                                );
                                continue;
                            }
                        }
                        Err(e) => {
                            eprintln!(
                                "DEBUG: Failed to parse JSON response: {} - line was: '{}'",
                                e, line
                            );
                            eprintln!("DEBUG: Raw bytes: {:?}", line.as_bytes());
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "DEBUG: Error reading from stdout: {} - collected {} lines so far",
                        e,
                        all_output.len()
                    );
                    if !all_output.is_empty() {
                        eprintln!("DEBUG: All stdout output before error: {:?}", all_output);
                    }
                    return Err(ProtocollieError::new(
                        ErrorCategory::Connection,
                        "READ_FAILED",
                        "Failed to read from MCP process stdout",
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        "Check if the MCP server process is still running",
                        "Verify the process stdout pipe is not broken",
                        "Try reconnecting to the server",
                    ]));
                }
            }
        }

        // Small delay to prevent busy waiting when we loop again
        if start_time.elapsed() < timeout {
            std::thread::sleep(Duration::from_millis(10));
        }

        eprintln!(
            "DEBUG: Timeout reached after {}ms - collected {} lines total",
            timeout_ms,
            all_output.len()
        );
        if !all_output.is_empty() {
            eprintln!(
                "DEBUG: All stdout output during timeout period: {:?}",
                all_output
            );
        }

        Err(
            ProtocollieError::connection_timeout("MCP server", timeout_ms).with_details(&format!(
                "Expected response with ID {} but received {} lines with no match",
                expected_id,
                all_output.len()
            )),
        )
    }

    /// Check if the process is still running
    pub fn check_process_status(&mut self) -> Result<bool, std::io::Error> {
        if let Some(child) = &mut self.process {
            match child.try_wait() {
                Ok(Some(_status)) => Ok(false), // Process has exited
                Ok(None) => Ok(true), // Process is still running
                Err(e) => Err(e), // Error checking status
            }
        } else {
            Ok(false) // No process
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut process) = self.process.take() {
            let _ = process.kill();
            let _ = process.wait();
        }
        self.stdin = None;
        self.stdout = None;
        eprintln!("DEBUG: Stopped MCP process for server {}", self.server_id);
    }
}

impl Drop for MCPProcess {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Global registry of MCP processes
pub static MCP_PROCESSES: Lazy<Arc<Mutex<HashMap<String, MCPProcess>>>> =
    Lazy::new(|| Arc::new(Mutex::new(HashMap::new())));

/// Start an MCP process for a specific server
pub async fn start_mcp_process(
    server_id: String,
    command: String,
    args: Vec<String>,
) -> Result<(), ProtocollieError> {
    eprintln!(
        "DEBUG: start_mcp_process called for server {} with command: {} {:?}",
        server_id, command, args
    );

    // Stop existing process if any (do this in separate scope to release mutex)
    {
        let mut processes = MCP_PROCESSES.lock().unwrap();
        if let Some(mut existing) = processes.remove(&server_id) {
            eprintln!("DEBUG: Stopping existing process for server {}", server_id);
            existing.stop();
        }
    }

    // Create and start new process
    let mut process = MCPProcess::new(server_id.clone());

    // Try to start the process
    if let Err(mut start_error) = process.start(&command, &args).await {
        // Collect any stderr that might explain the failure
        if let Some(stderr) = process.collect_stderr(1000) {
            start_error = start_error.with_details(&format!("Process stderr: {}", stderr));
        }
        return Err(start_error);
    }

    // Initialize the MCP connection
    eprintln!(
        "DEBUG: Initializing MCP connection for server {}",
        server_id
    );
    if let Err(mut init_error) = process.send_initialize() {
        // Wait a bit for any stderr to be captured
        std::thread::sleep(Duration::from_millis(500));

        // Collect stderr that might explain the initialization failure
        if let Some(stderr) = process.collect_stderr(2000) {
            init_error = init_error.with_details(&format!("Process stderr: {}", stderr));
        }
        return Err(init_error);
    }

    // Insert into processes map
    {
        let mut processes = MCP_PROCESSES.lock().unwrap();
        processes.insert(server_id.clone(), process);
    }

    eprintln!(
        "DEBUG: MCP process successfully started and initialized for server {}",
        server_id
    );
    Ok(())
}

/// Stop an MCP process for a specific server
pub fn stop_mcp_process(server_id: &str) {
    eprintln!("DEBUG: stop_mcp_process called for server {}", server_id);
    let mut processes = MCP_PROCESSES.lock().unwrap();
    if let Some(mut process) = processes.remove(server_id) {
        process.stop();
    }
}

/// List tools from a specific MCP server
pub fn list_mcp_tools(server_id: &str) -> Result<serde_json::Value, ProtocollieError> {
    eprintln!("DEBUG: list_mcp_tools called for server {}", server_id);

    let mut processes = MCP_PROCESSES.lock().unwrap();
    if let Some(process) = processes.get_mut(server_id) {
        // Check if the process is still running
        if let Some(child) = &mut process.process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    eprintln!(
                        "DEBUG: MCP process for server {} has exited with status: {:?}",
                        server_id, status
                    );
                    return Err(ProtocollieError::new(
                        ErrorCategory::Connection,
                        "PROCESS_EXITED",
                        &format!("MCP process for server {} has exited", server_id),
                    )
                    .with_details(&format!("Process exit status: {:?}", status))
                    .with_suggestions(vec![
                        "Check server logs for errors",
                        "Verify server configuration is correct",
                        "Try reconnecting to the server",
                    ]));
                }
                Ok(None) => {
                    eprintln!(
                        "DEBUG: MCP process for server {} is still running",
                        server_id
                    );
                }
                Err(e) => {
                    eprintln!(
                        "DEBUG: Error checking process status for server {}: {}",
                        server_id, e
                    );
                    return Err(ProtocollieError::new(
                        ErrorCategory::System,
                        "STATUS_CHECK_FAILED",
                        "Error checking MCP process status",
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        "Try reconnecting to the server",
                        "Restart the application if the issue persists",
                    ]));
                }
            }
        }

        let message_id = process.next_message_id();
        let list_tools_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": message_id,
            "method": "tools/list",
            "params": {}
        });

        // Send the message
        if let Err(e) = process.send_message_sync(list_tools_message) {
            return Err(e);
        }

        // Read the response
        match process.read_response(message_id as u64, 5000) {
            // 5 second timeout
            Ok(response) => {
                eprintln!(
                    "DEBUG: Got tools response for server {}: {}",
                    server_id, response
                );

                // Extract the result from the JSON-RPC response
                if let Some(result) = response.get("result") {
                    Ok(result.clone())
                } else if let Some(error) = response.get("error") {
                    Err(ProtocollieError::protocol_error(&format!(
                        "MCP server returned error: {}",
                        error
                    )))
                } else {
                    Err(ProtocollieError::protocol_error(
                        "Invalid JSON-RPC response: missing result and error",
                    ))
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        return Err(ProtocollieError::new(
            ErrorCategory::Connection,
            "NO_PROCESS",
            &format!("No active MCP process found for server {}", server_id),
        )
        .with_suggestions(vec![
            "Ensure the server is connected",
            "Try connecting to the server first",
            "Check that the server ID is correct",
        ]));
    }
}

/// Execute a tool on a specific MCP server
pub fn execute_mcp_tool(
    server_id: &str,
    tool_name: &str,
    arguments: serde_json::Value,
) -> Result<(serde_json::Value, u64), ProtocollieError> {
    eprintln!(
        "DEBUG: execute_mcp_tool called for server {} tool {} with args: {}",
        server_id, tool_name, arguments
    );

    let start_time = std::time::Instant::now();
    let mut processes = MCP_PROCESSES.lock().unwrap();
    if let Some(process) = processes.get_mut(server_id) {
        // Check if the process is still running
        if let Some(child) = &mut process.process {
            match child.try_wait() {
                Ok(Some(status)) => {
                    eprintln!(
                        "DEBUG: MCP process for server {} has exited with status: {:?}",
                        server_id, status
                    );
                    return Err(ProtocollieError::new(
                        ErrorCategory::Connection,
                        "PROCESS_EXITED",
                        &format!("MCP process for server {} has exited", server_id),
                    )
                    .with_details(&format!("Process exit status: {:?}", status))
                    .with_suggestions(vec![
                        "Check server logs for errors",
                        "Verify server configuration is correct",
                        "Try reconnecting to the server",
                    ]));
                }
                Ok(None) => {
                    eprintln!(
                        "DEBUG: MCP process for server {} is still running",
                        server_id
                    );
                }
                Err(e) => {
                    eprintln!(
                        "DEBUG: Error checking process status for server {}: {}",
                        server_id, e
                    );
                    return Err(ProtocollieError::new(
                        ErrorCategory::System,
                        "STATUS_CHECK_FAILED",
                        "Error checking MCP process status",
                    )
                    .with_details(&e.to_string())
                    .with_suggestions(vec![
                        "Try reconnecting to the server",
                        "Restart the application if the issue persists",
                    ]));
                }
            }
        }

        let message_id = process.next_message_id();
        let call_tool_message = serde_json::json!({
            "jsonrpc": "2.0",
            "id": message_id,
            "method": "tools/call",
            "params": {
                "name": tool_name,
                "arguments": arguments
            }
        });

        eprintln!("DEBUG: Sending tool call message: {}", call_tool_message);

        // Send the message
        if let Err(e) = process.send_message_sync(call_tool_message) {
            return Err(e);
        }

        // Read the response
        match process.read_response(message_id as u64, 10000) {
            // 10 second timeout for tool execution
            Ok(response) => {
                let duration_ms = start_time.elapsed().as_millis() as u64;
                eprintln!(
                    "DEBUG: Got tool response for server {} in {}ms: {}",
                    server_id, duration_ms, response
                );

                // Extract the result from the JSON-RPC response
                if let Some(result) = response.get("result") {
                    Ok((result.clone(), duration_ms))
                } else if let Some(error) = response.get("error") {
                    Err(ProtocollieError::new(
                        ErrorCategory::Protocol,
                        "TOOL_EXECUTION_ERROR",
                        &format!("Tool '{}' execution failed", tool_name),
                    )
                    .with_details(&format!("MCP server returned error: {}", error))
                    .with_suggestions(vec![
                        "Check the tool parameters are correct",
                        "Verify the tool exists on this server",
                        "Review server logs for more details",
                    ]))
                } else {
                    Err(ProtocollieError::protocol_error(
                        "Invalid JSON-RPC response: missing result and error",
                    ))
                }
            }
            Err(e) => {
                return Err(e);
            }
        }
    } else {
        return Err(ProtocollieError::new(
            ErrorCategory::Connection,
            "NO_PROCESS",
            &format!("No active MCP process found for server {}", server_id),
        )
        .with_suggestions(vec![
            "Ensure the server is connected",
            "Try connecting to the server first",
            "Check that the server ID is correct",
        ]));
    }
}

/// Check if a server has an active MCP process
pub fn is_mcp_process_running(server_id: &str) -> bool {
    let processes = MCP_PROCESSES.lock().unwrap();
    processes.contains_key(server_id)
}

/// Get connection status for all servers
pub fn get_all_server_connection_statuses() -> HashMap<String, bool> {
    let mut statuses = HashMap::new();
    let mut processes = MCP_PROCESSES.lock().unwrap();

    // Clone the keys to avoid borrowing issues
    let server_ids: Vec<String> = processes.keys().cloned().collect();

    for server_id in server_ids {
        if let Some(process) = processes.get_mut(&server_id) {
            // Check if the process is still running
            let is_running = if let Some(child) = &mut process.process {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        // Process has exited, remove it from registry
                        eprintln!("DEBUG: Removing dead process for server {}", server_id);
                        false
                    }
                    Ok(None) => true, // Still running
                    Err(_) => false,  // Error checking, assume dead
                }
            } else {
                false // No process
            };

            if is_running {
                statuses.insert(server_id.clone(), true);
            } else {
                // Remove dead process from registry
                drop(processes.get_mut(&server_id)); // Release the reference
                let mut processes_mut = MCP_PROCESSES.lock().unwrap();
                if let Some(mut dead_process) = processes_mut.remove(&server_id) {
                    dead_process.stop();
                }
                drop(processes_mut); // Release the lock
                statuses.insert(server_id, false);
                // Re-acquire lock for next iteration
                processes = MCP_PROCESSES.lock().unwrap();
            }
        }
    }

    statuses
}

/// Cleanup all MCP processes on application shutdown
pub fn cleanup_all_mcp_processes() {
    eprintln!("DEBUG: Cleaning up all MCP processes...");
    let mut processes = MCP_PROCESSES.lock().unwrap();
    let server_ids: Vec<String> = processes.keys().cloned().collect();

    for server_id in server_ids {
        eprintln!("DEBUG: Stopping MCP process for server {}", server_id);
        if let Some(mut process) = processes.remove(&server_id) {
            process.stop();
        }
    }

    eprintln!("DEBUG: All MCP processes cleaned up");
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MCPResponse {
    pub success: bool,
    pub data: Option<serde_json::Value>,
    pub error: Option<String>,
}