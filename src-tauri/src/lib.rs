mod domains;

use domains::terminal::manager::TerminalManager;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize domain managers
    let terminal_manager = TerminalManager::new();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(terminal_manager)
        .invoke_handler(tauri::generate_handler![
            greet,
            domains::terminal::create_terminal_process,
            domains::terminal::send_terminal_input,
            domains::terminal::execute_command,
            domains::terminal::kill_terminal_process,
            domains::terminal::get_terminal_processes,
            domains::terminal::get_terminal_process,
            domains::terminal::get_process_exit_code,
            domains::terminal::resize_terminal,
            domains::terminal::add_command_interceptor,
            domains::terminal::remove_command_interceptor,
            domains::terminal::add_output_parser,
            domains::terminal::remove_output_parser,
            domains::terminal::get_system_info
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
