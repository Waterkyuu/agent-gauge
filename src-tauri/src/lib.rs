mod adapters {
    pub(crate) mod activity;
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod process;
    pub(crate) mod workbuddy;
}
mod commands {
    pub(crate) mod activity;
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod comparison;
    pub(crate) mod workbuddy;
}
mod db {
    pub(crate) mod connection;
    pub(crate) mod migration;
}
mod dto {
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod comparison;
    pub(crate) mod workbuddy;
}
mod domain {
    pub(crate) mod agent_activity;
    pub(crate) mod agent_run;
    pub(crate) mod comparison;
}
mod error;
mod platform {
    pub(crate) mod claude_config;
    pub(crate) mod codex_config;
    pub(crate) mod process;
    pub(crate) mod workbuddy_config;
}
mod models {
    pub(crate) mod comparison;
}
mod repositories {
    pub(crate) mod comparison;
}
mod services {
    pub(crate) mod activity;
    pub(crate) mod agent;
    pub(crate) mod claude;
    pub(crate) mod codex;
    pub(crate) mod comparison;
    pub(crate) mod process;
    pub(crate) mod workbuddy;
}
mod utils {
    pub(crate) mod debounce;
}

use crate::adapters::activity::SystemAgentActivityAdapter;
use crate::adapters::claude::ClaudeRuntimeSettingsCache;
use crate::adapters::codex::CodexRuntimeDefaultsCache;
use crate::adapters::process::SystemAgentProcessAdapter;
use crate::commands::activity::AgentActivitiesResponse;
use crate::commands::agent::AgentProcessStatesResponse;
use crate::db::connection::connect_sqlite_path;
use crate::db::migration::Migrator;
use crate::platform::claude_config::{
    claude_settings_path, ClaudeConfigWatchEvent, ClaudeConfigWatcher,
};
use crate::platform::codex_config::{
    codex_config_paths, CodexConfigWatchEvent, CodexConfigWatcher,
};
use crate::platform::workbuddy_config::WorkBuddyConfigWatcherState;
use crate::repositories::comparison::ComparisonRepository;
use crate::services::activity::SystemAgentActivityMonitor;
use crate::services::comparison::ComparisonService;
use crate::services::process::AgentProcessMonitor;
use sea_orm_migration::MigratorTrait;
use std::time::Duration;
use tauri::{Emitter, Manager};

const AGENT_PROCESS_STATES_CHANGED_EVENT: &str = "agent-process-states-changed";
const AGENT_ACTIVITIES_CHANGED_EVENT: &str = "agent-activities-changed";
const AGENT_PROCESS_REFRESH_INTERVAL: Duration = Duration::from_secs(1);
const CODEX_CONFIG_CHANGED_EVENT: &str = "codex-config-changed";
const CLAUDE_CONFIG_CHANGED_EVENT: &str = "claude-config-changed";

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let claude_runtime_settings_cache = ClaudeRuntimeSettingsCache::default();
    let runtime_defaults_cache = CodexRuntimeDefaultsCache::default();
    tauri::Builder::default()
        .manage(claude_runtime_settings_cache)
        .manage(runtime_defaults_cache)
        .manage(WorkBuddyConfigWatcherState::default())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_data_directory = app.path().app_data_dir()?;
            let database_path = app_data_directory.join("agent-gauge.sqlite3");
            let comparison_database = tauri::async_runtime::block_on(async {
                tokio::fs::create_dir_all(&app_data_directory).await?;
                let database = connect_sqlite_path(&database_path)
                    .await
                    .map_err(std::io::Error::other)?;
                Migrator::up(&database, None)
                    .await
                    .map_err(std::io::Error::other)?;
                Ok::<_, std::io::Error>(database)
            })?;
            app.manage(ComparisonService::new(ComparisonRepository::new(
                comparison_database,
            )));

            let claude_runtime_settings_cache =
                app.state::<ClaudeRuntimeSettingsCache>().inner().clone();
            let runtime_defaults_cache = app.state::<CodexRuntimeDefaultsCache>().inner().clone();
            let main_window = app
                .get_webview_window("main")
                .ok_or_else(|| std::io::Error::other("main window is unavailable"))?;
            let process_window = main_window.clone();
            let activity_window = main_window.clone();
            let activity_monitor = SystemAgentActivityMonitor::start(
                SystemAgentActivityAdapter::default(),
                Default::default(),
                move |activities| {
                    // A closed window ends delivery while the retained monitor shuts down normally.
                    let _activity_event_delivered = activity_window
                        .emit(
                            AGENT_ACTIVITIES_CHANGED_EVENT,
                            AgentActivitiesResponse::from(activities),
                        )
                        .is_ok();
                },
            )
            .map_err(|_| std::io::Error::other("activity monitor failed to start"))?;
            let activity_handle = activity_monitor.handle();
            let process_activity_handle = activity_monitor.handle();
            let process_monitor = AgentProcessMonitor::start(
                SystemAgentProcessAdapter::default(),
                AGENT_PROCESS_REFRESH_INTERVAL,
                move |states| {
                    process_activity_handle.update_process_states(states);
                    // A closed window ends delivery while the retained monitor shuts down normally.
                    let _process_event_delivered = process_window
                        .emit(
                            AGENT_PROCESS_STATES_CHANGED_EVENT,
                            AgentProcessStatesResponse::from(states),
                        )
                        .is_ok();
                },
            )
            .map_err(|_| std::io::Error::other("process monitor failed to start"))?;
            activity_handle.update_process_states(
                process_monitor
                    .current_states()
                    .map_err(|_| std::io::Error::other("initial process snapshot unavailable"))?,
            );
            app.manage(activity_monitor);
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

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::activity::check_agent_activities,
            commands::agent::check_agent_processes,
            commands::claude::check_claude_login,
            commands::claude::run_claude_task,
            commands::codex::check_codex_login,
            commands::codex::run_codex_task,
            commands::comparison::get_comparison_history,
            commands::comparison::list_comparison_history,
            commands::comparison::save_comparison_history,
            commands::workbuddy::check_workbuddy_config,
            commands::workbuddy::check_workbuddy_login,
            commands::workbuddy::run_workbuddy_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
