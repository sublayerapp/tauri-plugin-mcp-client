use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub mod commands;
pub mod registry;
pub mod process;
pub mod error;

use registry::ConnectionRegistry;

/// Initialize the MCP plugin
pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("mcp")
        .setup(|app, _api| {
            // Initialize connection registry
            let mut registry = ConnectionRegistry::new();
            registry.set_app_handle(app.app_handle().clone());
            app.manage(registry);
            println!("MCP plugin initialized with connection registry and event system");
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::health_check,
            commands::get_connection_statuses,
            commands::plugin_connect_server,
            commands::plugin_disconnect_server,
            commands::plugin_list_tools,
            commands::plugin_execute_tool
        ])
        .build()
}