mod adapters {
    pub(crate) mod agent;
    pub(crate) mod codex;
}
mod commands {
    pub(crate) mod codex;
}
mod dto {
    pub(crate) mod codex;
}
mod domain {
    pub(crate) mod codex_run;
}
mod error;
mod services {
    pub(crate) mod agent;
    pub(crate) mod codex;
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::codex::check_codex_login,
            commands::codex::run_codex_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
