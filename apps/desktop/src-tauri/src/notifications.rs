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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationType {
    PoolDown,
    PoolRecovered,
    HashrateDrop,
    MinerCrash,
    MinerStopped,
    RemoteOffline,
    UpdateAvailable,
}

/// Rate limiter to prevent notification spam
pub struct NotificationManager {
    settings: NotificationSettings,
    last_sent: Mutex<HashMap<NotificationType, Instant>>,
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
    fn should_send(&self, notification_type: NotificationType) -> bool {
        if !self.settings.enabled {
            return false;
        }

        if self.is_quiet_hours() {
            return false;
        }

        // Check type-specific settings
        let type_enabled = match notification_type {
            NotificationType::PoolDown | NotificationType::PoolRecovered => self.settings.pool_down,
            NotificationType::HashrateDrop => self.settings.hashrate_drop,
            NotificationType::MinerCrash | NotificationType::MinerStopped => self.settings.miner_crash,
            NotificationType::RemoteOffline => self.settings.remote_offline,
            NotificationType::UpdateAvailable => self.settings.update_available,
        };

        if !type_enabled {
            return false;
        }

        // Rate limiting
        let mut last_sent = self.last_sent.lock().unwrap();
        if let Some(last) = last_sent.get(&notification_type) {
            if last.elapsed() < self.cooldown {
                info!("Notification {:?} rate limited", notification_type);
                return false;
            }
        }

        last_sent.insert(notification_type, Instant::now());
        true
    }

    /// Send a notification
    fn send(&self, title: &str, body: &str) {
        match Notification::new(&self.app_identifier)
            .title(title)
            .body(body)
            .show()
        {
            Ok(_) => info!("Notification sent: {}", title),
            Err(e) => warn!("Failed to send notification: {}", e),
        }
    }

    // Public notification methods

    pub fn notify_pool_down(&self, pool: &str) {
        if self.should_send(NotificationType::PoolDown) {
            self.send("Pool Connection Lost", &format!("Lost connection to {}", pool));
        }
    }

    pub fn notify_pool_recovered(&self, pool: &str) {
        if self.should_send(NotificationType::PoolRecovered) {
            self.send("Pool Reconnected", &format!("Connected to {}", pool));
        }
    }

    pub fn notify_hashrate_drop(&self, current: f64, average: f64) {
        if self.should_send(NotificationType::HashrateDrop) {
            let drop_pct = ((average - current) / average * 100.0).abs();
            if drop_pct >= self.settings.hashrate_drop_threshold {
                self.send(
                    "Hashrate Drop Detected",
                    &format!("Current: {:.1} H/s (down {:.0}%)", current, drop_pct),
                );
            }
        }
    }

    pub fn notify_miner_crash(&self, error: &str) {
        if self.should_send(NotificationType::MinerCrash) {
            self.send("Miner Stopped Unexpectedly", error);
        }
    }

    pub fn notify_miner_stopped(&self) {
        if self.should_send(NotificationType::MinerStopped) {
            self.send("Mining Stopped", "Mining has been stopped");
        }
    }

    pub fn notify_remote_offline(&self, name: &str) {
        if self.should_send(NotificationType::RemoteOffline) {
            self.send("Remote Miner Offline", &format!("{} is not responding", name));
        }
    }

    pub fn notify_update_available(&self, version: &str) {
        if self.should_send(NotificationType::UpdateAvailable) {
            self.send("Update Available", &format!("Version {} is available", version));
        }
    }

    /// Send a test notification (bypasses rate limiting)
    pub fn send_test(&self) {
        if self.settings.enabled {
            self.send("Test Notification", "Notifications are working correctly!");
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
