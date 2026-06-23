//! Alert inbox for tracking notifications (including suppressed ones).
//!
//! Stores alerts in a ring buffer so users can see events that occurred
//! during quiet hours or were deduplicated.

use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

const MAX_ALERTS: usize = 100;

static ALERT_ID_COUNTER: AtomicU64 = AtomicU64::new(1);

fn next_alert_id() -> u64 {
    ALERT_ID_COUNTER.fetch_add(1, Ordering::SeqCst)
}

/// Alert severity level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
}

/// Reason why an alert was suppressed
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuppressedReason {
    QuietHours,
    RateLimited,
    Deduplicated,
    NotificationsDisabled,
}

/// A stored alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: u64,
    pub timestamp: u64,
    pub alert_type: String,
    pub session_id: Option<String>,
    pub coin_symbol: Option<String>,
    pub message: String,
    pub severity: AlertSeverity,
    pub was_shown: bool,
    pub suppressed_reason: Option<SuppressedReason>,
}

/// Alert store with ring buffer
pub struct AlertStore {
    alerts: VecDeque<Alert>,
    max_size: usize,
}

impl AlertStore {
    pub fn new() -> Self {
        Self {
            alerts: VecDeque::with_capacity(MAX_ALERTS),
            max_size: MAX_ALERTS,
        }
    }

    /// Record a new alert
    pub fn record(
        &mut self,
        alert_type: &str,
        session_id: Option<&str>,
        coin_symbol: Option<&str>,
        message: &str,
        severity: AlertSeverity,
        was_shown: bool,
        suppressed_reason: Option<SuppressedReason>,
    ) -> Alert {
        let alert = Alert {
            id: next_alert_id(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            alert_type: alert_type.to_string(),
            session_id: session_id.map(|s| s.to_string()),
            coin_symbol: coin_symbol.map(|s| s.to_string()),
            message: message.to_string(),
            severity,
            was_shown,
            suppressed_reason,
        };

        // Remove oldest if at capacity
        if self.alerts.len() >= self.max_size {
            self.alerts.pop_front();
        }
        
        self.alerts.push_back(alert.clone());
        alert
    }

    /// Record a shown alert
    pub fn record_shown(
        &mut self,
        alert_type: &str,
        session_id: Option<&str>,
        coin_symbol: Option<&str>,
        message: &str,
        severity: AlertSeverity,
    ) -> Alert {
        self.record(alert_type, session_id, coin_symbol, message, severity, true, None)
    }

    /// Record a suppressed alert
    pub fn record_suppressed(
        &mut self,
        alert_type: &str,
        session_id: Option<&str>,
        coin_symbol: Option<&str>,
        message: &str,
        severity: AlertSeverity,
        reason: SuppressedReason,
    ) -> Alert {
        self.record(alert_type, session_id, coin_symbol, message, severity, false, Some(reason))
    }

    /// List alerts (newest first)
    pub fn list(&self, limit: usize, since_id: Option<u64>) -> Vec<Alert> {
        let iter = self.alerts.iter().rev();
        
        let filtered: Vec<_> = if let Some(since) = since_id {
            iter.filter(|a| a.id > since).take(limit).cloned().collect()
        } else {
            iter.take(limit).cloned().collect()
        };
        
        filtered
    }

    /// Get unread count (suppressed alerts)
    pub fn unread_count(&self) -> usize {
        self.alerts.iter().filter(|a| !a.was_shown).count()
    }

    /// Clear all alerts
    pub fn clear(&mut self) {
        self.alerts.clear();
    }

    /// Mark all as read
    pub fn mark_all_read(&mut self) {
        for alert in &mut self.alerts {
            alert.was_shown = true;
        }
    }
}

impl Default for AlertStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alert_store_basic() {
        let mut store = AlertStore::new();
        
        let alert = store.record_shown(
            "pool_down",
            Some("sess1"),
            Some("XMR"),
            "Pool connection lost",
            AlertSeverity::Warning,
        );
        
        assert!(alert.id > 0);
        assert!(alert.was_shown);
        assert_eq!(store.unread_count(), 0);
    }

    #[test]
    fn test_suppressed_alert() {
        let mut store = AlertStore::new();
        
        store.record_suppressed(
            "hashrate_drop",
            Some("sess1"),
            Some("VRSC"),
            "Hashrate dropped 50%",
            AlertSeverity::Warning,
            SuppressedReason::QuietHours,
        );
        
        assert_eq!(store.unread_count(), 1);
        
        let alerts = store.list(10, None);
        assert_eq!(alerts.len(), 1);
        assert!(!alerts[0].was_shown);
    }

    #[test]
    fn test_ring_buffer() {
        let mut store = AlertStore::new();
        store.max_size = 5;
        
        for i in 0..10 {
            store.record_shown(
                "test",
                None,
                None,
                &format!("Alert {}", i),
                AlertSeverity::Info,
            );
        }
        
        let alerts = store.list(10, None);
        assert_eq!(alerts.len(), 5);
        // Should have alerts 5-9 (newest)
        assert!(alerts[0].message.contains("9"));
    }

    #[test]
    fn test_list_since_id() {
        let mut store = AlertStore::new();
        
        let a1 = store.record_shown("t", None, None, "1", AlertSeverity::Info);
        let _a2 = store.record_shown("t", None, None, "2", AlertSeverity::Info);
        let _a3 = store.record_shown("t", None, None, "3", AlertSeverity::Info);
        
        let alerts = store.list(10, Some(a1.id));
        assert_eq!(alerts.len(), 2); // a2 and a3
    }
}
