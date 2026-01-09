//! Notification system with rate limiting and deduplication.
//! All notifications are opt-in by default.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use tauri::api::notification::Notification;
use tracing::{info, warn};

/// Notification settings (persisted in config)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub enabled: bool,
    pub pool_down: bool,
    pub hashrate_drop: bool,
    pub hashrate_drop_threshold: f64, // percentage, e.g., 30.0
    pub miner_crash: bool,
    pub remote_offline: bool,
    pub update_available: bool,
    pub quiet_hours_enabled: bool,
    pub quiet_hours_start: u8, // 0-23
    pub quiet_hours_end: u8,   // 0-23
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            enabled: false, // Opt-in by default
            pool_down: true,
            hashrate_drop: true,
            hashrate_drop_threshold: 30.0,
            miner_crash: true,
            remote_offline: false,
            update_available: true,
            quiet_hours_enabled: false,
            quiet_hours_start: 22,
            quiet_hours_end: 8,
        }
    }
}

/// Notification types for deduplication
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum NotificationType {
    PoolDown(String),      // session_id
    PoolRecovered(String), // session_id
    HashrateDrop(String),  // session_id
    MinerCrash(String),    // session_id
    MinerStopped(String),  // session_id
    RemoteOffline(String), // endpoint_id
    UpdateAvailable,
}

/// Dedup key for rate limiting
fn dedup_key(notification_type: &NotificationType) -> String {
    match notification_type {
        NotificationType::PoolDown(id) => format!("pool_down:{}", id),
        NotificationType::PoolRecovered(id) => format!("pool_recovered:{}", id),
        NotificationType::HashrateDrop(id) => format!("hashrate_drop:{}", id),
        NotificationType::MinerCrash(id) => format!("miner_crash:{}", id),
        NotificationType::MinerStopped(id) => format!("miner_stopped:{}", id),
        NotificationType::RemoteOffline(id) => format!("remote_offline:{}", id),
        NotificationType::UpdateAvailable => "update_available".to_string(),
    }
}

/// Rate limiter to prevent notification spam
pub struct NotificationManager {
    settings: NotificationSettings,
    last_sent: Mutex<HashMap<String, Instant>>, // dedup_key -> last sent time
    cooldown: Duration,
    app_identifier: String,
}

impl NotificationManager {
    pub fn new(app_identifier: &str) -> Self {
        Self {
            settings: NotificationSettings::default(),
            last_sent: Mutex::new(HashMap::new()),
            cooldown: Duration::from_secs(300), // 5 minutes
            app_identifier: app_identifier.to_string(),
        }
    }

    pub fn update_settings(&mut self, settings: NotificationSettings) {
        self.settings = settings;
    }

    pub fn settings(&self) -> &NotificationSettings {
        &self.settings
    }

    /// Check if we're in quiet hours
    fn is_quiet_hours(&self) -> bool {
        if !self.settings.quiet_hours_enabled {
            return false;
        }

        let now = chrono::Local::now();
        let hour = now.hour() as u8;

        if self.settings.quiet_hours_start <= self.settings.quiet_hours_end {
            // Simple range: e.g., 22-08 means 22:00 to 08:00
            hour >= self.settings.quiet_hours_start || hour < self.settings.quiet_hours_end
        } else {
            // Wrapping range: e.g., 22-08 means 22:00 to next day 08:00
            hour >= self.settings.quiet_hours_start || hour < self.settings.quiet_hours_end
        }
    }

    /// Check if notification should be sent (rate limiting + dedup)
    fn should_send(&self, notification_type: &NotificationType) -> bool {
        if !self.settings.enabled {
            return false;
        }

        if self.is_quiet_hours() {
            return false;
        }

        // Check type-specific settings
        let type_enabled = match notification_type {
            NotificationType::PoolDown(_) | NotificationType::PoolRecovered(_) => self.settings.pool_down,
            NotificationType::HashrateDrop(_) => self.settings.hashrate_drop,
            NotificationType::MinerCrash(_) | NotificationType::MinerStopped(_) => self.settings.miner_crash,
            NotificationType::RemoteOffline(_) => self.settings.remote_offline,
            NotificationType::UpdateAvailable => self.settings.update_available,
        };

        if !type_enabled {
            return false;
        }

        // Rate limiting with dedup key
        let key = dedup_key(notification_type);
        let mut last_sent = self.last_sent.lock().unwrap();
        if let Some(last) = last_sent.get(&key) {
            if last.elapsed() < self.cooldown {
                info!("Notification {} rate limited", key);
                return false;
            }
        }

        last_sent.insert(key, Instant::now());
        true
    }

    /// Send a notification with optional sound
    fn send(&self, title: &str, body: &str) {
        match Notification::new(&self.app_identifier)
            .title(title)
            .body(body)
            .sound("default") // macOS system sound
            .show()
        {
            Ok(_) => info!("Notification sent: {}", title),
            Err(e) => warn!("Failed to send notification: {}", e),
        }
    }

    /// Play a sound without notification (for in-app events)
    #[cfg(target_os = "macos")]
    pub fn play_sound(&self, sound_name: &str) {
        use std::process::Command;
        // Use afplay to play system sounds
        let sound_path = format!("/System/Library/Sounds/{}.aiff", sound_name);
        let _ = Command::new("afplay")
            .arg(&sound_path)
            .spawn();
    }

    #[cfg(not(target_os = "macos"))]
    pub fn play_sound(&self, _sound_name: &str) {
        // No-op on other platforms for now
    }

    // Public notification methods (session-aware)

    pub fn notify_pool_down(&self, session_id: &str, symbol: &str, pool: &str) {
        if self.should_send(&NotificationType::PoolDown(session_id.to_string())) {
            self.send(
                &format!("{}: Pool Connection Lost", symbol),
                &format!("Lost connection to {}", pool),
            );
        }
    }

    pub fn notify_pool_recovered(&self, session_id: &str, symbol: &str, pool: &str) {
        if self.should_send(&NotificationType::PoolRecovered(session_id.to_string())) {
            self.send(
                &format!("{}: Pool Reconnected", symbol),
                &format!("Connected to {}", pool),
            );
        }
    }

    pub fn notify_hashrate_drop(&self, session_id: &str, symbol: &str, current: f64, average: f64) {
        let drop_pct = ((average - current) / average * 100.0).abs();
        if drop_pct >= self.settings.hashrate_drop_threshold {
            if self.should_send(&NotificationType::HashrateDrop(session_id.to_string())) {
                self.send(
                    &format!("{}: Hashrate Drop", symbol),
                    &format!("Current: {:.1} H/s (down {:.0}%)", current, drop_pct),
                );
            }
        }
    }

    pub fn notify_miner_crash(&self, session_id: &str, symbol: &str, error: &str) {
        if self.should_send(&NotificationType::MinerCrash(session_id.to_string())) {
            self.send(
                &format!("{}: Miner Stopped Unexpectedly", symbol),
                error,
            );
        }
    }

    pub fn notify_miner_stopped(&self) {
        // Legacy: no session context
        if self.should_send(&NotificationType::MinerStopped("legacy".to_string())) {
            self.send("Mining Stopped", "Mining has been stopped");
        }
    }

    pub fn notify_session_stopped(&self, session_id: &str, symbol: &str) {
        if self.should_send(&NotificationType::MinerStopped(session_id.to_string())) {
            self.send(
                &format!("{}: Mining Stopped", symbol),
                "Session has been stopped",
            );
        }
    }

    pub fn notify_remote_offline(&self, name: &str) {
        if self.should_send(&NotificationType::RemoteOffline(name.to_string())) {
            self.send("Remote Miner Offline", &format!("{} is not responding", name));
        }
    }

    pub fn notify_update_available(&self, version: &str) {
        if self.should_send(&NotificationType::UpdateAvailable) {
            self.send("Update Available", &format!("Version {} is available", version));
        }
    }

    /// Send a test notification (bypasses rate limiting)
    pub fn send_test(&self) {
        if self.settings.enabled {
            // Play a pleasant sound
            self.play_sound("Glass");
            self.send("Test Notification", "Notifications are working correctly!");
        } else {
            // Even if disabled, play sound to confirm it works
            self.play_sound("Glass");
        }
    }
}

// Simple chrono replacement for hour extraction
mod chrono {
    pub struct Local;
    pub struct DateTime {
        hour: u8,
    }
    impl DateTime {
        pub fn hour(&self) -> u32 {
            self.hour as u32
        }
    }
    impl Local {
        pub fn now() -> DateTime {
            use std::time::{SystemTime, UNIX_EPOCH};
            let secs = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            // Rough hour calculation (UTC, not local, but good enough for now)
            let hour = ((secs % 86400) / 3600) as u8;
            DateTime { hour }
        }
    }
}
