use crate::notifications::{NotificationManager, NotificationSettings};
use crate::tray;
use openminedash_core::{
    AppState, CoinDefinition, CrashRecoveryState, MiningConfig, MiningStatus, Profile,
    create_diagnostics_export,
};
use openminedash_pools::PoolHealthResult;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

type AppStateHandle = Arc<Mutex<AppState>>;
type NotificationHandle = Arc<Mutex<NotificationManager>>;

#[tauri::command]
pub async fn get_consent(state: State<'_, AppStateHandle>) -> Result<bool, String> {
    let state = state.lock().await;
    Ok(state.has_consent())
}

#[tauri::command]
pub async fn set_consent(state: State<'_, AppStateHandle>, consent: bool) -> Result<(), String> {
    let mut state = state.lock().await;
    state.set_consent(consent);
    state.save_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_theme(state: State<'_, AppStateHandle>) -> Result<String, String> {
    let state = state.lock().await;
    Ok(state.theme().to_string())
}

#[tauri::command]
pub async fn set_theme(_state: State<'_, AppStateHandle>, _theme: String) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
pub async fn get_custom_binary_path(state: State<'_, AppStateHandle>) -> Result<Option<String>, String> {
    let state = state.lock().await;
    Ok(state.custom_binary_path().map(|p| p.display().to_string()))
}

#[tauri::command]
pub async fn set_custom_binary_path(
    state: State<'_, AppStateHandle>,
    path: Option<String>,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.set_custom_binary_path(path.map(PathBuf::from));
    state.save_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_coins(state: State<'_, AppStateHandle>) -> Result<Vec<CoinDefinition>, String> {
    let state = state.lock().await;
    state.list_coins().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn start_mining(
    state: State<'_, AppStateHandle>,
    config: MiningConfig,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut state = state.lock().await;
    if !state.has_consent() {
        return Err("Mining consent not granted".to_string());
    }
    
    state.start_mining(config, app_handle.clone()).await.map_err(|e| e.to_string())?;
    
    // Update tray
    let status = state.status();
    tray::update_tray(&app_handle, true, status.hashrate, status.accepted_shares, status.uptime, "balanced");
    
    Ok(())
}

#[tauri::command]
pub async fn stop_mining(
    state: State<'_, AppStateHandle>,
    notifications: State<'_, NotificationHandle>,
    app_handle: tauri::AppHandle,
) -> Result<(), String> {
    let mut state = state.lock().await;
    state.stop_mining().await.map_err(|e| e.to_string())?;
    
    // Update tray
    tray::update_tray(&app_handle, false, 0.0, 0, 0, "balanced");
    
    // Notify
    let notif = notifications.lock().await;
    notif.notify_miner_stopped();
    
    Ok(())
}

#[tauri::command]
pub async fn get_status(
    state: State<'_, AppStateHandle>,
    app_handle: tauri::AppHandle,
) -> Result<MiningStatus, String> {
    let mut state = state.lock().await;
    let _ = state.refresh_stats().await;
    let status = state.status().clone();
    
    // Update tray with latest stats
    if status.is_running {
        tray::update_tray(
            &app_handle,
            true,
            status.hashrate,
            status.accepted_shares,
            status.uptime,
            "balanced",
        );
    }
    
    Ok(status)
}

#[tauri::command]
pub async fn save_profile(state: State<'_, AppStateHandle>, profile: Profile) -> Result<(), String> {
    let mut state = state.lock().await;
    state.save_profile(profile);
    state.save_config().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_profiles(state: State<'_, AppStateHandle>) -> Result<Vec<Profile>, String> {
    let state = state.lock().await;
    Ok(state.profiles().to_vec())
}

#[tauri::command]
pub async fn check_pool_health(url: String) -> Result<PoolHealthResult, String> {
    openminedash_pools::check_health(&url).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn export_diagnostics(
    state: State<'_, AppStateHandle>,
    mask_wallets: bool,
) -> Result<String, String> {
    let _state = state.lock().await;
    let config = openminedash_core::AppConfig::load().unwrap_or_default();
    let logs = Vec::new();
    let export = create_diagnostics_export(&config, logs, mask_wallets);
    serde_json::to_string_pretty(&export).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_crash_recovery_state(
    state: State<'_, AppStateHandle>,
) -> Result<CrashRecoveryState, String> {
    let state = state.lock().await;
    Ok(state.crash_recovery_state().clone())
}

#[tauri::command]
pub async fn clear_crash_recovery(state: State<'_, AppStateHandle>) -> Result<(), String> {
    let mut state = state.lock().await;
    state.clear_crash_recovery();
    Ok(())
}

// Notification commands

#[tauri::command]
pub async fn get_notification_settings(
    notifications: State<'_, NotificationHandle>,
) -> Result<NotificationSettings, String> {
    let notif = notifications.lock().await;
    Ok(notif.settings().clone())
}

#[tauri::command]
pub async fn set_notification_settings(
    notifications: State<'_, NotificationHandle>,
    settings: NotificationSettings,
) -> Result<(), String> {
    let mut notif = notifications.lock().await;
    notif.update_settings(settings);
    Ok(())
}

#[tauri::command]
pub async fn send_test_notification(
    notifications: State<'_, NotificationHandle>,
) -> Result<(), String> {
    let notif = notifications.lock().await;
    notif.send_test();
    Ok(())
}

#[tauri::command]
pub async fn update_tray_state(
    app_handle: tauri::AppHandle,
    is_running: bool,
    hashrate: f64,
    accepted: u64,
    uptime: u64,
    preset: String,
) -> Result<(), String> {
    tray::update_tray(&app_handle, is_running, hashrate, accepted, uptime, &preset);
    Ok(())
}
