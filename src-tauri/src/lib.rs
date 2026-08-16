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
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod workbuddy;
}
mod domain {
    pub(crate) mod agent_run;
}
mod error;
mod platform {
    pub(crate) mod claude_config;
    pub(crate) mod codex_config;
    pub(crate) mod process;
    pub(crate) mod workbuddy_config;
}
mod services {
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod process;
    pub(crate) mod workbuddy;
}
mod utils {
    pub(crate) mod debounce;
}

use crate::adapters::claude::ClaudeRuntimeSettingsCache;
use crate::adapters::codex::CodexRuntimeDefaultsCache;
use crate::adapters::process::SystemAgentProcessAdapter;
use crate::adapters::workbuddy::{read_workbuddy_config, workbuddy_local_storage_path};
use crate::commands::agent::AgentProcessStatesResponse;
use crate::dto::workbuddy::WorkBuddyConfigStatus;
use crate::platform::claude_config::{
    claude_settings_path, ClaudeConfigWatchEvent, ClaudeConfigWatcher,
};
use crate::platform::codex_config::{
    codex_config_paths, CodexConfigWatchEvent, CodexConfigWatcher,
};
use crate::platform::workbuddy_config::{WorkBuddyConfigWatchEvent, WorkBuddyConfigWatcher};
use crate::services::process::AgentProcessMonitor;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{Emitter, Manager};

const AGENT_PROCESS_STATES_CHANGED_EVENT: &str = "agent-process-states-changed";
const AGENT_PROCESS_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const CODEX_CONFIG_CHANGED_EVENT: &str = "codex-config-changed";
const CLAUDE_CONFIG_CHANGED_EVENT: &str = "claude-config-changed";
const WORKBUDDY_CONFIG_CHANGED_EVENT: &str = "workbuddy-config-changed";

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
            let process_window = main_window.clone();
            let process_monitor = AgentProcessMonitor::start(
                SystemAgentProcessAdapter::default(),
                AGENT_PROCESS_REFRESH_INTERVAL,
                move |states| {
                    let _ = process_window.emit(
                        AGENT_PROCESS_STATES_CHANGED_EVENT,
                        AgentProcessStatesResponse::from(states),
                    );
                },
            )
            .map_err(|_| std::io::Error::other("process monitor failed to start"))?;
            app.manage(process_monitor);

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

            if let Some(local_storage_path) = workbuddy_local_storage_path() {
                let callback_window = main_window.clone();
                let previous_config = Arc::new(Mutex::new(None::<WorkBuddyConfigStatus>));
                let watcher = WorkBuddyConfigWatcher::start(local_storage_path, move |event| {
                    if event == WorkBuddyConfigWatchEvent::Failed {
                        if let Ok(mut previous) = previous_config.lock() {
                            *previous = None;
                        }
                        return;
                    }

                    let Ok(config) = read_workbuddy_config().map(WorkBuddyConfigStatus::from)
                    else {
                        return;
                    };
                    let Ok(mut previous) = previous_config.lock() else {
                        return;
                    };
                    if previous.as_ref() == Some(&config) {
                        return;
                    }
                    *previous = Some(config.clone());
                    drop(previous);

                    if callback_window
                        .emit(WORKBUDDY_CONFIG_CHANGED_EVENT, config)
                        .is_err()
                    {
                        // The page may not be mounted yet; its initial snapshot command still
                        // supplies the current configuration when it starts listening.
                    }
                });

                if let Ok(Some(watcher)) = watcher {
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
            commands::workbuddy::check_workbuddy_config,
            commands::workbuddy::check_workbuddy_login,
            commands::workbuddy::run_workbuddy_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
