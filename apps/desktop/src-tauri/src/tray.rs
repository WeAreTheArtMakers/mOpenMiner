//! System tray (menu bar) implementation for macOS.
//! Single source of truth - created only in Rust, not JS.

use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu,
};
use tracing::info;

/// Session info for tray display
#[derive(Debug, Clone)]
pub struct TraySessionInfo {
    pub id: String,
    pub symbol: String,
    pub hashrate: f64,
    pub status: String,
}

/// Build the system tray with initial stopped state
pub fn create_tray() -> SystemTray {
    let menu = build_tray_menu(false, 0.0, 0, 0, "balanced", &[]);
    SystemTray::new().with_menu(menu)
}

/// Build tray menu based on current state
pub fn build_tray_menu(
    is_running: bool,
    hashrate: f64,
    accepted: u64,
    uptime: u64,
    preset: &str,
    sessions: &[TraySessionInfo],
) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new();

    // Status header
    let status_text = if !sessions.is_empty() {
        let total_hashrate: f64 = sessions.iter().map(|s| s.hashrate).sum();
        format!("● {} Session{} - {:.1} H/s", 
            sessions.len(),
            if sessions.len() > 1 { "s" } else { "" },
            total_hashrate
        )
    } else if is_running {
        format!("● RUNNING - {:.1} H/s", hashrate)
    } else {
        "○ STOPPED".to_string()
    };
    menu = menu.add_item(CustomMenuItem::new("status", status_text).disabled());

    // KPI line when running (legacy)
    if is_running && sessions.is_empty() {
        let kpi = format!("Accepted: {} | Uptime: {}s", accepted, uptime);
        menu = menu.add_item(CustomMenuItem::new("kpi", kpi).disabled());
    }

    menu = menu.add_native_item(SystemTrayMenuItem::Separator);

    // STOP ALL - always first when any session is running
    if !sessions.is_empty() || is_running {
        menu = menu.add_item(
            CustomMenuItem::new("stop_all", "⏹ Stop All Mining")
                .accelerator("CmdOrCtrl+.")
        );
        menu = menu.add_native_item(SystemTrayMenuItem::Separator);
    }

    // Session list (max 6)
    if !sessions.is_empty() {
        let display_sessions: Vec<_> = sessions.iter().take(6).collect();
        for session in &display_sessions {
            let hashrate_str = if session.hashrate > 0.0 {
                format!("{:.1} H/s", session.hashrate)
            } else {
                "—".to_string()
            };
            let label = format!("{} · {} · {}", 
                session.symbol, 
                hashrate_str,
                session.status.to_uppercase()
            );
            
            // Session submenu
            let session_menu = SystemTrayMenu::new()
                .add_item(CustomMenuItem::new(format!("session_stop_{}", session.id), "Stop"))
                .add_item(CustomMenuItem::new(format!("session_logs_{}", session.id), "Open Logs"));
            
            menu = menu.add_submenu(SystemTraySubmenu::new(label, session_menu));
        }
        
        if sessions.len() > 6 {
            let more = format!("+{} more → Open Dashboard", sessions.len() - 6);
            menu = menu.add_item(CustomMenuItem::new("dashboard", more));
        }
        
        menu = menu.add_native_item(SystemTrayMenuItem::Separator);
    }

    // Start mining (only when nothing running)
    if sessions.is_empty() && !is_running {
        menu = menu.add_item(CustomMenuItem::new("start", "▶ Start Mining"));
        menu = menu.add_native_item(SystemTrayMenuItem::Separator);
    }

    // Preset submenu
    let preset_menu = SystemTrayMenu::new()
        .add_item(preset_item("eco", "Eco (~25% CPU)", preset))
        .add_item(preset_item("balanced", "Balanced (~50% CPU)", preset))
        .add_item(preset_item("max", "Max (~75% CPU)", preset));
    menu = menu.add_submenu(SystemTraySubmenu::new("Performance", preset_menu));

    menu = menu.add_native_item(SystemTrayMenuItem::Separator);

    // Navigation
    menu = menu
        .add_item(CustomMenuItem::new("dashboard", "Open Dashboard").accelerator("CmdOrCtrl+D"))
        .add_item(CustomMenuItem::new("logs", "Open Logs").accelerator("CmdOrCtrl+L"));

    menu = menu.add_native_item(SystemTrayMenuItem::Separator);

    // Quit
    menu = menu.add_item(CustomMenuItem::new("quit", "Quit").accelerator("CmdOrCtrl+Q"));

    menu
}

fn preset_item(id: &str, label: &str, current: &str) -> CustomMenuItem {
    let display = if id == current {
        format!("✓ {}", label)
    } else {
        format!("  {}", label)
    };
    CustomMenuItem::new(format!("preset_{}", id), display)
}

/// Handle tray events
pub fn handle_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick { .. } => {
            // Toggle main window visibility
            if let Some(window) = app.get_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => {
            info!("Tray menu clicked: {}", id);
            match id.as_str() {
                "stop" | "stop_all" => {
                    let _ = app.emit_all("tray-action", "stop_all");
                }
                "start" => {
                    let _ = app.emit_all("tray-action", "start");
                }
                "dashboard" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit_all("tray-action", "navigate:dashboard");
                }
                "logs" => {
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit_all("tray-action", "navigate:logs");
                }
                "quit" => {
                    // Emit quit event so app can clean up (stop mining)
                    let _ = app.emit_all("tray-action", "quit");
                }
                id if id.starts_with("preset_") => {
                    let preset = id.trim_start_matches("preset_");
                    let _ = app.emit_all("tray-action", format!("preset:{}", preset));
                }
                id if id.starts_with("session_stop_") => {
                    let session_id = id.trim_start_matches("session_stop_");
                    let _ = app.emit_all("tray-action", format!("session_stop:{}", session_id));
                }
                id if id.starts_with("session_logs_") => {
                    let session_id = id.trim_start_matches("session_logs_");
                    if let Some(window) = app.get_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                    let _ = app.emit_all("tray-action", format!("session_logs:{}", session_id));
                }
                _ => {}
            }
        }
        _ => {}
    }
}

/// Update tray menu with new state (called from state changes)
pub fn update_tray(
    app: &AppHandle,
    is_running: bool,
    hashrate: f64,
    accepted: u64,
    uptime: u64,
    preset: &str,
) {
    let menu = build_tray_menu(is_running, hashrate, accepted, uptime, preset, &[]);
    if let Some(tray) = app.tray_handle_by_id("main") {
        let _ = tray.set_menu(menu);
    }
}

/// Update tray with session info
pub fn update_tray_with_sessions(
    app: &AppHandle,
    sessions: Vec<TraySessionInfo>,
    preset: &str,
) {
    let is_running = !sessions.is_empty();
    let total_hashrate: f64 = sessions.iter().map(|s| s.hashrate).sum();
    let menu = build_tray_menu(is_running, total_hashrate, 0, 0, preset, &sessions);
    if let Some(tray) = app.tray_handle_by_id("main") {
        let _ = tray.set_menu(menu);
    }
}
