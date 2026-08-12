mod adapters {
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod workbuddy;
}
mod commands {
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod workbuddy;
}
mod dto {
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod workbuddy;
}
mod domain {
    pub(crate) mod codex_run;
}
mod error;
mod services {
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod workbuddy;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::claude::check_claude_login,
            commands::claude::run_claude_task,
            commands::codex::check_codex_login,
            commands::codex::run_codex_task,
            commands::workbuddy::check_workbuddy_login,
            commands::workbuddy::run_workbuddy_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
