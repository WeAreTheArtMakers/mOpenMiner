#![cfg_attr(
    all(not(debug_assertions), target_os = "macos"),
    windows_subsystem = "windows"
)]

mod commands;
mod notifications;
mod tray;

use commands::*;
use notifications::NotificationManager;
use openminedash_core::AppState;
use std::sync::Arc;
use tokio::sync::Mutex;

fn main() {
    tracing_subscriber::fmt::init();

    let state = Arc::new(Mutex::new(AppState::new()));
    let notification_manager = Arc::new(Mutex::new(
        NotificationManager::new("com.openminedash.app")
    ));

    tauri::Builder::default()
        .system_tray(tray::create_tray())
        .on_system_tray_event(|app, event| tray::handle_tray_event(app, event))
        .manage(state)
        .manage(notification_manager)
        .invoke_handler(tauri::generate_handler![
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
            list_profiles,
            check_pool_health,
            export_diagnostics,
            get_crash_recovery_state,
            clear_crash_recovery,
            get_notification_settings,
            set_notification_settings,
            send_test_notification,
            update_tray_state,
        ])
        .setup(|_app| {
            // Tray is ready - no additional setup needed
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
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
