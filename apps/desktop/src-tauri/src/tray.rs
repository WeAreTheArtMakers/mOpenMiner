//! System tray (menu bar) implementation for macOS.
//! Single source of truth - created only in Rust, not JS.

use tauri::{
    AppHandle, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, SystemTraySubmenu,
};
use tracing::info;

/// Build the system tray with initial stopped state
pub fn create_tray() -> SystemTray {
    let menu = build_tray_menu(false, 0.0, 0, 0, "balanced");
    SystemTray::new().with_menu(menu)
}

/// Build tray menu based on current state
pub fn build_tray_menu(
    is_running: bool,
    hashrate: f64,
    accepted: u64,
    uptime: u64,
    preset: &str,
) -> SystemTrayMenu {
    let mut menu = SystemTrayMenu::new();

    // Status header
    let status_text = if is_running {
        format!("● RUNNING - {:.1} H/s", hashrate)
    } else {
        "○ STOPPED".to_string()
    };
    menu = menu.add_item(CustomMenuItem::new("status", status_text).disabled());

    // KPI line when running
    if is_running {
        let kpi = format!("Accepted: {} | Uptime: {}s", accepted, uptime);
        menu = menu.add_item(CustomMenuItem::new("kpi", kpi).disabled());
    }

    menu = menu.add_native_item(SystemTrayMenuItem::Separator);

    // Primary action - STOP always first when running (danger action)
    if is_running {
        menu = menu.add_item(
            CustomMenuItem::new("stop", "⏹ Stop Mining")
                .accelerator("CmdOrCtrl+.")
        );
    } else {
        menu = menu.add_item(CustomMenuItem::new("start", "▶ Start Mining"));
    }

    menu = menu.add_native_item(SystemTrayMenuItem::Separator);

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
                "stop" => {
                    let _ = app.emit_all("tray-action", "stop");
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
    let menu = build_tray_menu(is_running, hashrate, accepted, uptime, preset);
    if let Some(tray) = app.tray_handle_by_id("main") {
        let _ = tray.set_menu(menu);
    }
}
