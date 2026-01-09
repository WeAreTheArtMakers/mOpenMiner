#![cfg_attr(
    all(not(debug_assertions), target_os = "macos"),
    windows_subsystem = "windows"
)]

mod commands;
mod notifications;
mod tray;

use commands::*;
use notifications::NotificationManager;
use openminedash_core::{AppState, SessionManager, AlertStore, AppConfig};
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(Mutex::new(AppState::new()));
    let notification_manager = Arc::new(Mutex::new(
        NotificationManager::new("com.openminedash.app")
    ));
    let session_manager = Arc::new(Mutex::new(SessionManager::new()));
    let alert_store = Arc::new(Mutex::new(AlertStore::new()));

    // Clone for quit handler
    let session_manager_quit = session_manager.clone();

    tauri::Builder::default()
        .system_tray(tray::create_tray())
        .on_system_tray_event(|app, event| tray::handle_tray_event(app, event))
        .manage(state)
        .manage(notification_manager)
        .manage(session_manager.clone())
        .manage(alert_store)
        .invoke_handler(tauri::generate_handler![
            // Legacy commands (backward compatibility)
            get_consent,
            set_consent,
            get_theme,
            set_theme,
            get_custom_binary_path,
            set_custom_binary_path,
            list_coins,
            start_mining,
            stop_mining,
            get_status,
            save_profile,
            delete_profile,
            list_profiles,
            check_pool_health,
            fetch_pool_balance,
            export_diagnostics,
            get_crash_recovery_state,
            clear_crash_recovery,
            get_notification_settings,
            set_notification_settings,
            send_test_notification,
            play_notification_sound,
            update_tray_state,
            // Session management commands
            start_session,
            stop_session,
            suspend_session,
            resume_session,
            list_sessions,
            get_session,
            get_session_logs,
            stop_all_sessions,
            get_active_session_count,
            refresh_session_stats,
            // Alert commands
            list_alerts,
            get_unread_alert_count,
            mark_alerts_read,
            clear_alerts,
            // Thread budget commands
            get_thread_budget_settings,
            set_thread_budget_settings,
            get_budget_status,
            // Mining history commands
            get_mining_history,
            get_history_summary,
            clear_mining_history,
        ])
        .setup(move |app| {
            // Set app handle for session manager
            let handle = app.handle();
            let sm = session_manager.clone();
            tauri::async_runtime::spawn(async move {
                let mut manager = sm.lock().await;
                manager.set_app_handle(handle);
            });
            Ok(())
        })
        .on_window_event(|event| {
            // Handle window close - hide instead of quit (menu bar app behavior)
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                #[cfg(target_os = "macos")]
                {
                    event.window().hide().unwrap();
                    api.prevent_close();
                }
            }
        })
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(move |_app_handle, event| {
            // Handle app quit - stop all sessions if configured
            if let tauri::RunEvent::ExitRequested { .. } = event {
                let config = AppConfig::load().unwrap_or_default();
                if config.behavior.quit_stops_mining {
                    let sm = session_manager_quit.clone();
                    tauri::async_runtime::block_on(async {
                        let manager = sm.lock().await;
                        let _ = manager.stop_all().await;
                    });
                }
            }
        });
}
