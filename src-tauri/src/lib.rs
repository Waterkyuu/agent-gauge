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
    pub(crate) mod claude_config;
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

use crate::adapters::claude::ClaudeRuntimeSettingsCache;
use crate::adapters::codex::CodexRuntimeDefaultsCache;
use crate::platform::claude_config::{
    claude_settings_path, ClaudeConfigWatchEvent, ClaudeConfigWatcher,
};
use crate::platform::codex_config::{
    codex_config_paths, CodexConfigWatchEvent, CodexConfigWatcher,
};
use tauri::{Emitter, Manager};

const CODEX_CONFIG_CHANGED_EVENT: &str = "codex-config-changed";
const CLAUDE_CONFIG_CHANGED_EVENT: &str = "claude-config-changed";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let claude_runtime_settings_cache = ClaudeRuntimeSettingsCache::default();
    let runtime_defaults_cache = CodexRuntimeDefaultsCache::default();
    tauri::Builder::default()
        .manage(claude_runtime_settings_cache)
        .manage(runtime_defaults_cache)
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let claude_runtime_settings_cache =
                app.state::<ClaudeRuntimeSettingsCache>().inner().clone();
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

            if let Some(settings_path) = claude_settings_path() {
                let callback_cache = claude_runtime_settings_cache.clone();
                let callback_window = main_window.clone();
                let watcher = ClaudeConfigWatcher::start(settings_path, move |event| {
                    match event {
                        ClaudeConfigWatchEvent::Changed => callback_cache.invalidate(),
                        ClaudeConfigWatchEvent::Failed => callback_cache.disable(),
                    }

                    if callback_window
                        .emit(CLAUDE_CONFIG_CHANGED_EVENT, ())
                        .is_err()
                    {
                        callback_cache.disable();
                    }
                });

                if let Ok(Some(watcher)) = watcher {
                    claude_runtime_settings_cache.enable();
                    app.manage(watcher);
                }
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
