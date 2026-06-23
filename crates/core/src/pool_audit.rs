//! Pool Audit Trail - Güvenli ve takip edilebilir pool değişiklik log'u
//!
//! Her pool ekleme/silme işlemi:
//! 1. Zaman damgası ile loglanır
//! 2. Hangi coin'de yapıldığı kaydedilir
//! 3. Eski ve yeni state karşılaştırılır
//! 4. Audit JSON dosyasına yazılır

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PoolAction {
    #[serde(rename = "add")]
    Add,
    #[serde(rename = "remove")]
    Remove,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolAuditEntry {
    /// ISO 8601 timestamp
    pub timestamp: String,
    /// Unix timestamp for sorting
    pub unix_ts: u64,
    /// Coin ID (e.g. "btc", "xmr")
    pub coin_id: String,
    /// Action: add or remove
    pub action: PoolAction,
    /// Pool name
    pub pool_name: String,
    /// Pool stratum URL
    pub pool_url: String,
    /// Previous pool count before this action
    pub previous_count: usize,
    /// New pool count after this action
    pub new_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolAuditLog {
    pub entries: Vec<PoolAuditEntry>,
}

impl PoolAuditLog {
    /// Load audit log from disk
    pub fn load() -> Self {
        let path = Self::audit_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    serde_json::from_str(&content).unwrap_or_default()
                }
                Err(e) => {
                    tracing::warn!("Failed to read audit log: {}", e);
                    Self::default()
                }
            }
        } else {
            Self::default()
        }
    }

    /// Save audit log to disk
    pub fn save(&self) {
        let path = Self::audit_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        match serde_json::to_string_pretty(self) {
            Ok(content) => {
                if let Err(e) = std::fs::write(&path, content) {
                    tracing::error!("Failed to save audit log: {}", e);
                }
            }
            Err(e) => {
                tracing::error!("Failed to serialize audit log: {}", e);
            }
        }
    }

    /// Add a new entry to the audit log
    pub fn add_entry(
        &mut self,
        coin_id: &str,
        action: PoolAction,
        pool_name: &str,
        pool_url: &str,
        previous_count: usize,
        new_count: usize,
    ) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();

        let entry = PoolAuditEntry {
            timestamp: chrono_now(),
            unix_ts: now.as_secs(),
            coin_id: coin_id.to_string(),
            action,
            pool_name: pool_name.to_string(),
            pool_url: pool_url.to_string(),
            previous_count,
            new_count,
        };

        self.entries.push(entry);
        self.save();
    }

    /// Get recent entries (newest first)
    pub fn recent(&self, limit: usize) -> Vec<&PoolAuditEntry> {
        let mut sorted: Vec<_> = self.entries.iter().collect();
        sorted.sort_by(|a, b| b.unix_ts.cmp(&a.unix_ts));
        sorted.truncate(limit);
        sorted
    }

    /// Get entries for a specific coin
    pub fn for_coin(&self, coin_id: &str) -> Vec<&PoolAuditEntry> {
        self.entries
            .iter()
            .filter(|e| e.coin_id == coin_id)
            .collect()
    }

    /// Get the audit file path
    fn audit_path() -> PathBuf {
        // Use data directory for persistent storage
        dirs::data_dir()
            .map(|d| d.join("openminedash").join("pool_audit.json"))
            .unwrap_or_else(|| PathBuf::from("pool_audit.json"))
    }
}

/// Simple chrono-like timestamp without chrono dependency
fn chrono_now() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = now.as_secs();
    
    // Calculate UTC date/time components
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let hours = time_secs / 3600;
    let minutes = (time_secs % 3600) / 60;
    let seconds = time_secs % 60;
    
    // Simple date calculation from Unix epoch
    let mut y = 1970i64;
    let mut remaining_days = days as i64;
    
    loop {
        let days_in_year = if is_leap_year(y) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        y += 1;
    }
    
    let months_days: &[i64] = if is_leap_year(y) {
        &[31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        &[31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };
    
    let mut m = 0;
    for (i, &md) in months_days.iter().enumerate() {
        if remaining_days < md {
            m = i + 1;
            break;
        }
        remaining_days -= md;
    }
    if m == 0 { m = 12; }
    
    let d = remaining_days + 1;
    
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        y, m, d, hours, minutes, seconds
    )
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_entry_creation() {
        let mut log = PoolAuditLog::default();
        log.add_entry("btc", PoolAction::Add, "TestPool", "stratum+tcp://test.com:3333", 3, 4);
        assert_eq!(log.entries.len(), 1);
        assert_eq!(log.entries[0].coin_id, "btc");
        assert_eq!(log.entries[0].previous_count, 3);
        assert_eq!(log.entries[0].new_count, 4);
    }

    #[test]
    fn test_audit_recent() {
        let mut log = PoolAuditLog::default();
        log.add_entry("btc", PoolAction::Add, "Pool1", "url1", 1, 2);
        log.add_entry("xmr", PoolAction::Add, "Pool2", "url2", 2, 3);
        log.add_entry("btc", PoolAction::Remove, "Pool1", "url1", 2, 1);
        
        let recent = log.recent(2);
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].coin_id, "btc"); // newest first: remove btc
        assert_eq!(recent[0].action as u8, PoolAction::Remove as u8);
    }
}