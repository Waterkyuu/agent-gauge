mod adapters {
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod process;
    pub(crate) mod workbuddy;
}
mod commands {
    pub(crate) mod agent;
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
mod platform {
    pub(crate) mod codex_config;
    pub(crate) mod process;
}
mod services {
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod process;
    pub(crate) mod workbuddy;
}

use crate::adapters::codex::CodexRuntimeDefaultsCache;
use crate::platform::codex_config::{
    codex_config_paths, CodexConfigWatchEvent, CodexConfigWatcher,
};
use tauri::{Emitter, Manager};

const CODEX_CONFIG_CHANGED_EVENT: &str = "codex-config-changed";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let runtime_defaults_cache = CodexRuntimeDefaultsCache::default();
    tauri::Builder::default()
        .manage(runtime_defaults_cache)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let runtime_defaults_cache = app.state::<CodexRuntimeDefaultsCache>().inner().clone();
            let main_window = app
                .get_webview_window("main")
                .ok_or_else(|| std::io::Error::other("main window is unavailable"))?;
            let callback_cache = runtime_defaults_cache.clone();
            let callback_window = main_window.clone();
            let watcher = CodexConfigWatcher::start(codex_config_paths(), move |event| {
                match event {
                    CodexConfigWatchEvent::Changed => callback_cache.invalidate(),
                    CodexConfigWatchEvent::Failed => callback_cache.disable(),
                }

                if callback_window
                    .emit(CODEX_CONFIG_CHANGED_EVENT, ())
                    .is_err()
                {
                    callback_cache.disable();
                }
            });

            if let Ok(Some(watcher)) = watcher {
                runtime_defaults_cache.enable();
                app.manage(watcher);
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::agent::check_agent_processes,
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
