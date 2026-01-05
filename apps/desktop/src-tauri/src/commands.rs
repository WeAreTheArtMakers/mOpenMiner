use crate::notifications::{NotificationManager, NotificationSettings};
use crate::tray;
use openminedash_core::{
    AppState, CoinDefinition, CrashRecoveryState, MiningConfig, MiningStatus, Profile,
    SessionConfig, SessionDetails, SessionManager, SessionSummary, LogsResponse,
    create_diagnostics_export, Alert, AlertSeverity, AlertStore, BudgetStatus,
    ThreadBudgetSettings, calculate_budget, BudgetMode, BudgetPreset,
};
use openminedash_pools::PoolHealthResult;
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{Manager, State};
use tokio::sync::Mutex;

type AppStateHandle = Arc<Mutex<AppState>>;
type NotificationHandle = Arc<Mutex<NotificationManager>>;
type SessionManagerHandle = Arc<Mutex<SessionManager>>;
type AlertStoreHandle = Arc<Mutex<AlertStore>>;

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

// ============================================================================
// Session Management Commands (Multi-session mining)
// ============================================================================

#[tauri::command]
pub async fn start_session(
    sessions: State<'_, SessionManagerHandle>,
    config: SessionConfig,
) -> Result<String, String> {
    let manager = sessions.lock().await;
    manager.start_session(config).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn stop_session(
    sessions: State<'_, SessionManagerHandle>,
    session_id: String,
) -> Result<(), String> {
    let manager = sessions.lock().await;
    manager.stop_session(&session_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn suspend_session(
    sessions: State<'_, SessionManagerHandle>,
    session_id: String,
) -> Result<(), String> {
    let manager = sessions.lock().await;
    manager.suspend_session(&session_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn resume_session(
    sessions: State<'_, SessionManagerHandle>,
    session_id: String,
) -> Result<(), String> {
    let manager = sessions.lock().await;
    manager.resume_session(&session_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn list_sessions(
    sessions: State<'_, SessionManagerHandle>,
) -> Result<Vec<SessionSummary>, String> {
    let manager = sessions.lock().await;
    Ok(manager.list_sessions().await)
}

#[tauri::command]
pub async fn get_session(
    sessions: State<'_, SessionManagerHandle>,
    session_id: String,
) -> Result<Option<SessionDetails>, String> {
    let manager = sessions.lock().await;
    Ok(manager.get_session(&session_id).await)
}

#[tauri::command]
pub async fn get_session_logs(
    sessions: State<'_, SessionManagerHandle>,
    session_id: String,
    cursor: Option<u64>,
    limit: Option<usize>,
) -> Result<Option<LogsResponse>, String> {
    let manager = sessions.lock().await;
    Ok(manager.get_session_logs(&session_id, cursor, limit).await)
}

#[tauri::command]
pub async fn stop_all_sessions(
    sessions: State<'_, SessionManagerHandle>,
) -> Result<(), String> {
    let manager = sessions.lock().await;
    manager.stop_all().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_active_session_count(
    sessions: State<'_, SessionManagerHandle>,
) -> Result<usize, String> {
    let manager = sessions.lock().await;
    Ok(manager.active_count().await)
}

#[tauri::command]
pub async fn refresh_session_stats(
    sessions: State<'_, SessionManagerHandle>,
) -> Result<Vec<SessionSummary>, String> {
    let manager = sessions.lock().await;
    manager.refresh_all_stats().await;
    Ok(manager.list_sessions().await)
}

// ============================================================================
// Alert Inbox Commands
// ============================================================================

#[tauri::command]
pub async fn list_alerts(
    alerts: State<'_, AlertStoreHandle>,
    limit: Option<usize>,
    since_id: Option<u64>,
) -> Result<Vec<Alert>, String> {
    let store = alerts.lock().await;
    Ok(store.list(limit.unwrap_or(50), since_id))
}

#[tauri::command]
pub async fn get_unread_alert_count(
    alerts: State<'_, AlertStoreHandle>,
) -> Result<usize, String> {
    let store = alerts.lock().await;
    Ok(store.unread_count())
}

#[tauri::command]
pub async fn mark_alerts_read(
    alerts: State<'_, AlertStoreHandle>,
) -> Result<(), String> {
    let mut store = alerts.lock().await;
    store.mark_all_read();
    Ok(())
}

#[tauri::command]
pub async fn clear_alerts(
    alerts: State<'_, AlertStoreHandle>,
) -> Result<(), String> {
    let mut store = alerts.lock().await;
    store.clear();
    Ok(())
}

// ============================================================================
// Thread Budget Commands
// ============================================================================

#[tauri::command]
pub async fn get_thread_budget_settings(
    state: State<'_, AppStateHandle>,
) -> Result<ThreadBudgetSettings, String> {
    let state = state.lock().await;
    let config = openminedash_core::AppConfig::load().unwrap_or_default();
    Ok(config.thread_budget)
}

#[tauri::command]
pub async fn set_thread_budget_settings(
    state: State<'_, AppStateHandle>,
    settings: ThreadBudgetSettings,
) -> Result<(), String> {
    let _state = state.lock().await;
    let mut config = openminedash_core::AppConfig::load().unwrap_or_default();
    config.thread_budget = settings;
    config.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_budget_status(
    sessions: State<'_, SessionManagerHandle>,
) -> Result<BudgetStatus, String> {
    let manager = sessions.lock().await;
    let sessions_list = manager.list_sessions().await;
    
    let active_count = sessions_list.iter()
        .filter(|s| s.stats.status == openminedash_core::SessionStatus::Running)
        .count() as u32;
    
    let total_threads: u32 = sessions_list.iter()
        .filter(|s| s.stats.status == openminedash_core::SessionStatus::Running)
        .map(|s| s.config.threads_hint)
        .sum();
    
    let config = openminedash_core::AppConfig::load().unwrap_or_default();
    Ok(calculate_budget(&config.thread_budget, active_count, total_threads))
}
