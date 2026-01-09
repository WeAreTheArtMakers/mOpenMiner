//! Mining history persistence
//! Stores mining session history to disk for tracking earnings over time

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

/// A completed mining session record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningRecord {
    pub id: String,
    pub coin: String,
    pub symbol: String,
    pub pool: String,
    pub wallet: String,
    pub worker: String,
    pub started_at: u64,      // Unix timestamp
    pub ended_at: u64,        // Unix timestamp
    pub duration_secs: u64,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub avg_hashrate: f64,
    pub algorithm: String,
}

/// Mining history store
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningHistory {
    pub records: Vec<MiningRecord>,
    pub total_time_secs: u64,
    pub total_accepted_shares: u64,
    pub total_rejected_shares: u64,
}

impl MiningHistory {
    /// Load history from disk
    pub fn load() -> Self {
        let path = Self::history_path();
        if path.exists() {
            match std::fs::read_to_string(&path) {
                Ok(content) => {
                    match serde_json::from_str(&content) {
                        Ok(history) => {
                            info!("Loaded mining history with {} records", 
                                  Self::record_count(&history));
                            return history;
                        }
                        Err(e) => {
                            warn!("Failed to parse mining history: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to read mining history: {}", e);
                }
            }
        }
        Self::default()
    }

    fn record_count(history: &MiningHistory) -> usize {
        history.records.len()
    }

    /// Save history to disk
    pub fn save(&self) -> Result<(), std::io::Error> {
        let path = Self::history_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        info!("Saved mining history with {} records", self.records.len());
        Ok(())
    }

    /// Add a completed mining session
    pub fn add_record(&mut self, record: MiningRecord) {
        self.total_time_secs += record.duration_secs;
        self.total_accepted_shares += record.accepted_shares;
        self.total_rejected_shares += record.rejected_shares;
        self.records.push(record);
        let _ = self.save();
    }

    /// Get records for a specific coin
    pub fn records_by_coin(&self, coin: &str) -> Vec<&MiningRecord> {
        self.records.iter().filter(|r| r.coin == coin).collect()
    }

    /// Get records within a time range
    pub fn records_in_range(&self, start: u64, end: u64) -> Vec<&MiningRecord> {
        self.records.iter()
            .filter(|r| r.started_at >= start && r.started_at <= end)
            .collect()
    }

    /// Get summary statistics
    pub fn get_summary(&self) -> HistorySummary {
        let mut by_coin: std::collections::HashMap<String, CoinSummary> = std::collections::HashMap::new();
        
        for record in &self.records {
            let entry = by_coin.entry(record.coin.clone()).or_insert(CoinSummary {
                coin: record.coin.clone(),
                symbol: record.symbol.clone(),
                total_time_secs: 0,
                total_accepted: 0,
                total_rejected: 0,
                session_count: 0,
                wallets: Vec::new(),
            });
            
            entry.total_time_secs += record.duration_secs;
            entry.total_accepted += record.accepted_shares;
            entry.total_rejected += record.rejected_shares;
            entry.session_count += 1;
            
            if !entry.wallets.contains(&record.wallet) {
                entry.wallets.push(record.wallet.clone());
            }
        }

        HistorySummary {
            total_sessions: self.records.len(),
            total_time_secs: self.total_time_secs,
            total_accepted_shares: self.total_accepted_shares,
            total_rejected_shares: self.total_rejected_shares,
            by_coin: by_coin.into_values().collect(),
        }
    }

    /// Clear all history
    pub fn clear(&mut self) {
        self.records.clear();
        self.total_time_secs = 0;
        self.total_accepted_shares = 0;
        self.total_rejected_shares = 0;
        let _ = self.save();
    }

    fn history_path() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openminer")
            .join("mining_history.json")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistorySummary {
    pub total_sessions: usize,
    pub total_time_secs: u64,
    pub total_accepted_shares: u64,
    pub total_rejected_shares: u64,
    pub by_coin: Vec<CoinSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinSummary {
    pub coin: String,
    pub symbol: String,
    pub total_time_secs: u64,
    pub total_accepted: u64,
    pub total_rejected: u64,
    pub session_count: usize,
    pub wallets: Vec<String>,
}
