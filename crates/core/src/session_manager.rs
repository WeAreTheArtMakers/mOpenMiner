//! Multi-session mining manager.
//!
//! Manages multiple concurrent mining sessions with:
//! - Per-session stats, logs, and lifecycle
//! - Thread-safe access via Arc<RwLock>
//! - Event emission for UI updates (throttled 1Hz stats, batched logs)
//! - Crash recovery support

use crate::{route_algorithm, CoreError, MinerType, Result};
use openminedash_miner_adapters::{
    CpuminerOptAdapter, MiningConfig as AdapterMiningConfig, PerformancePreset,
    XMRigAdapter,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Manager;
use tokio::process::Child;
use tokio::sync::RwLock;
use tracing::{error, info};
use uuid::Uuid;

/// Event throttling constants
const STATS_THROTTLE_MS: u64 = 1000; // 1Hz stats updates
const LOG_BATCH_SIZE: usize = 20;    // Batch logs in chunks

pub type SessionId = String;

/// Session configuration (user-provided, non-secret)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    pub coin_id: String,
    pub symbol: String,
    pub algorithm: String,
    pub miner_kind: MinerKind,
    pub pool_url: String,
    pub wallet: String,
    pub worker: String,
    pub preset: PerformancePreset,
    pub threads_hint: u32,
    pub created_at: u64,
    /// Stable identity hash for this config
    #[serde(default)]
    pub config_hash: String,
}

impl SessionConfig {
    /// Generate stable config hash
    pub fn compute_hash(&self) -> String {
        let pool_host = self.pool_url
            .split("://").last().unwrap_or(&self.pool_url)
            .split(':').next().unwrap_or(&self.pool_url);
        let wallet_prefix = if self.wallet.len() >= 8 {
            &self.wallet[..8]
        } else {
            &self.wallet
        };
        
        let input = format!(
            "{}|{}|{:?}|{}|{}|{}|{:?}|{}",
            self.coin_id,
            self.algorithm,
            self.miner_kind,
            pool_host,
            wallet_prefix,
            self.worker,
            self.preset,
            self.threads_hint
        );
        
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        hex::encode(&hasher.finalize()[..8]) // First 8 bytes = 16 hex chars
    }
    
    /// Get pool host for display
    pub fn pool_host(&self) -> String {
        self.pool_url
            .split("://").last().unwrap_or(&self.pool_url)
            .split(':').next().unwrap_or(&self.pool_url)
            .to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MinerKind {
    XMRig,
    CpuminerOpt,
}

impl std::fmt::Display for MinerKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MinerKind::XMRig => write!(f, "xmrig"),
            MinerKind::CpuminerOpt => write!(f, "cpuminer-opt"),
        }
    }
}

/// Session status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SessionStatus {
    Stopped,
    Starting,
    Running,
    Suspended,
    Stopping,
    Error,
}

impl Default for SessionStatus {
    fn default() -> Self {
        Self::Stopped
    }
}

/// Telemetry confidence level
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TelemetryConfidence {
    High,   // API-based stats (XMRig HTTP)
    Medium, // Log parsing with good patterns
    Low,    // Log parsing with limited patterns
    #[default]
    Unknown,
}

/// Connection state (best-effort from logs)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectionState {
    Connecting,
    Connected,
    Subscribed,
    Authorized,
    #[default]
    Unknown,
}

/// Real-time session statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionStats {
    pub status: SessionStatus,
    pub hashrate_current: f64,
    pub hashrate_avg60: f64,
    pub accepted: u64,
    pub rejected: u64,
    pub difficulty: f64,
    pub last_share_time: Option<u64>,
    pub uptime_secs: u64,
    pub connected: bool,
    pub last_error: Option<String>,
    /// Confidence level for parsed stats (0.0-1.0) - legacy
    pub stats_confidence: f64,
    /// Telemetry confidence level
    #[serde(default)]
    pub telemetry_confidence: TelemetryConfidence,
    /// Reason for telemetry confidence
    #[serde(default)]
    pub telemetry_reason: String,
    /// Connection state
    #[serde(default)]
    pub connection_state: ConnectionState,
    /// Thread budget overcommit flag
    #[serde(default)]
    pub overcommitted: bool,
    /// Overcommit ratio (1.0 = at budget, >1.0 = over)
    #[serde(default)]
    pub overcommit_ratio: f32,
}

/// Summary for list_sessions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: SessionId,
    pub config: SessionConfig,
    pub stats: SessionStats,
}

/// Full session details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDetails {
    pub id: SessionId,
    pub config: SessionConfig,
    pub stats: SessionStats,
}


/// Log entry with cursor support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub line: String,
}

/// Log response with pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogsResponse {
    pub session_id: SessionId,
    pub lines: Vec<LogEntry>,
    pub next_cursor: Option<u64>,
    pub has_more: bool,
}

/// Ring buffer for logs (bounded memory)
const LOG_BUFFER_SIZE: usize = 500;

struct LogBuffer {
    entries: Vec<LogEntry>,
    cursor: u64,
}

impl LogBuffer {
    fn new() -> Self {
        Self {
            entries: Vec::with_capacity(LOG_BUFFER_SIZE),
            cursor: 0,
        }
    }

    fn push(&mut self, line: String) {
        let entry = LogEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as u64,
            line,
        };
        
        if self.entries.len() >= LOG_BUFFER_SIZE {
            self.entries.remove(0);
        }
        self.entries.push(entry);
        self.cursor += 1;
    }

    fn get_logs(&self, from_cursor: Option<u64>, limit: usize) -> LogsResponse {
        let start_idx = if let Some(cursor) = from_cursor {
            let total = self.cursor;
            let buffer_start = total.saturating_sub(self.entries.len() as u64);
            if cursor < buffer_start {
                0
            } else {
                (cursor - buffer_start) as usize
            }
        } else {
            0
        };

        let entries: Vec<_> = self.entries
            .iter()
            .skip(start_idx)
            .take(limit)
            .cloned()
            .collect();

        let has_more = start_idx + entries.len() < self.entries.len();
        let next_cursor = if has_more {
            Some(self.cursor - (self.entries.len() - start_idx - entries.len()) as u64)
        } else {
            None
        };

        LogsResponse {
            session_id: String::new(), // filled by caller
            lines: entries,
            next_cursor,
            has_more,
        }
    }
}

/// Internal session runtime state (not serialized)
struct SessionRuntime {
    child: Option<Child>,
    xmrig_adapter: Option<XMRigAdapter>,
    cpuminer_adapter: Option<CpuminerOptAdapter>,
    logs: LogBuffer,
    start_time: u64,
    /// Last stats emit timestamp (for throttling)
    last_stats_emit: u64,
    /// Pending log lines (for batching)
    pending_logs: Vec<String>,
}

/// A mining session
struct MiningSession {
    id: SessionId,
    config: SessionConfig,
    stats: SessionStats,
    runtime: SessionRuntime,
}

impl MiningSession {
    fn new(id: SessionId, mut config: SessionConfig) -> Self {
        // Compute config hash if not set
        if config.config_hash.is_empty() {
            config.config_hash = config.compute_hash();
        }
        Self {
            id,
            config,
            stats: SessionStats::default(),
            runtime: SessionRuntime {
                child: None,
                xmrig_adapter: None,
                cpuminer_adapter: None,
                logs: LogBuffer::new(),
                start_time: 0,
                last_stats_emit: 0,
                pending_logs: Vec::new(),
            },
        }
    }

    fn to_summary(&self) -> SessionSummary {
        SessionSummary {
            id: self.id.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }

    fn to_details(&self) -> SessionDetails {
        SessionDetails {
            id: self.id.clone(),
            config: self.config.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Thread-safe session manager
pub struct SessionManager {
    sessions: Arc<RwLock<HashMap<SessionId, MiningSession>>>,
    app_handle: Option<tauri::AppHandle>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: tauri::AppHandle) {
        self.app_handle = Some(handle);
    }

    fn emit_event(&self, event: &str, payload: impl Serialize + Clone) {
        if let Some(handle) = &self.app_handle {
            let _ = handle.emit_all(event, payload);
        }
    }

    /// Start a new mining session
    pub async fn start_session(&self, config: SessionConfig) -> Result<SessionId> {
        let id = Uuid::new_v4().to_string();
        
        // Route to appropriate miner
        let routing = route_algorithm(&config.algorithm, true);
        let miner_kind = match routing.miner_type {
            MinerType::XMRig => MinerKind::XMRig,
            MinerType::CpuminerOpt => MinerKind::CpuminerOpt,
            _ => return Err(CoreError::Miner(
                routing.warning.unwrap_or_else(|| "Algorithm not supported".to_string())
            )),
        };

        let mut session_config = config;
        session_config.miner_kind = miner_kind;
        session_config.created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let mut session = MiningSession::new(id.clone(), session_config.clone());
        session.stats.status = SessionStatus::Starting;

        // Create adapter config
        let adapter_config = AdapterMiningConfig {
            coin: session_config.algorithm.clone(),
            pool: session_config.pool_url.clone(),
            wallet: session_config.wallet.clone(),
            worker: session_config.worker.clone(),
            threads: session_config.threads_hint,
            preset: session_config.preset,
        };

        // Start the miner
        let app_handle = self.app_handle.clone()
            .ok_or_else(|| CoreError::Miner("App handle not set".to_string()))?;

        let child = match miner_kind {
            MinerKind::XMRig => {
                let mut adapter = XMRigAdapter::new();
                let child = adapter.start(&adapter_config, app_handle).await
                    .map_err(|e| CoreError::Miner(e.to_string()))?;
                session.runtime.xmrig_adapter = Some(adapter);
                child
            }
            MinerKind::CpuminerOpt => {
                let mut adapter = CpuminerOptAdapter::new();
                let child = adapter.start(&adapter_config, app_handle).await
                    .map_err(|e| CoreError::Miner(e.to_string()))?;
                session.runtime.cpuminer_adapter = Some(adapter);
                child
            }
        };

        session.runtime.child = Some(child);
        session.runtime.start_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        session.stats.status = SessionStatus::Running;
        session.stats.connected = true;

        // Store session
        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(id.clone(), session);
        }

        // Emit event
        self.emit_event("session://created", serde_json::json!({
            "session_id": id,
            "config": session_config,
        }));

        info!("Started session {} for {}", id, session_config.symbol);
        Ok(id)
    }

    /// Stop a session
    pub async fn stop_session(&self, session_id: &str) -> Result<()> {
        let mut sessions = self.sessions.write().await;
        
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| CoreError::Miner(format!("Session not found: {}", session_id)))?;

        if session.stats.status != SessionStatus::Running && 
           session.stats.status != SessionStatus::Suspended {
            return Ok(());
        }

        session.stats.status = SessionStatus::Stopping;

        if let Some(mut child) = session.runtime.child.take() {
            match session.config.miner_kind {
                MinerKind::XMRig => {
                    if let Some(adapter) = &mut session.runtime.xmrig_adapter {
                        adapter.stop(&mut child).await;
                    }
                }
                MinerKind::CpuminerOpt => {
                    if let Some(adapter) = &mut session.runtime.cpuminer_adapter {
                        adapter.stop(&mut child).await;
                    }
                }
            }
        }

        session.stats.status = SessionStatus::Stopped;
        session.stats.connected = false;

        let symbol = session.config.symbol.clone();
        
        // Emit event
        self.emit_event("session://stopped", serde_json::json!({
            "session_id": session_id,
            "symbol": symbol,
        }));

        info!("Stopped session {} ({})", session_id, symbol);
        Ok(())
    }

    /// Suspend a session (SIGSTOP)
    #[cfg(unix)]
    pub async fn suspend_session(&self, session_id: &str) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| CoreError::Miner(format!("Session not found: {}", session_id)))?;

        if session.stats.status != SessionStatus::Running {
            return Err(CoreError::InvalidState);
        }

        if let Some(child) = &session.runtime.child {
            if let Some(pid) = child.id() {
                kill(Pid::from_raw(pid as i32), Signal::SIGSTOP)
                    .map_err(|e| CoreError::Miner(format!("Failed to suspend: {}", e)))?;
                session.stats.status = SessionStatus::Suspended;
                
                self.emit_event("session://updated", serde_json::json!({
                    "session_id": session_id,
                    "status": "suspended",
                }));
                
                info!("Suspended session {}", session_id);
            }
        }

        Ok(())
    }

    /// Resume a suspended session (SIGCONT)
    #[cfg(unix)]
    pub async fn resume_session(&self, session_id: &str) -> Result<()> {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        let mut sessions = self.sessions.write().await;
        let session = sessions.get_mut(session_id)
            .ok_or_else(|| CoreError::Miner(format!("Session not found: {}", session_id)))?;

        if session.stats.status != SessionStatus::Suspended {
            return Err(CoreError::InvalidState);
        }

        if let Some(child) = &session.runtime.child {
            if let Some(pid) = child.id() {
                kill(Pid::from_raw(pid as i32), Signal::SIGCONT)
                    .map_err(|e| CoreError::Miner(format!("Failed to resume: {}", e)))?;
                session.stats.status = SessionStatus::Running;
                
                self.emit_event("session://updated", serde_json::json!({
                    "session_id": session_id,
                    "status": "running",
                }));
                
                info!("Resumed session {}", session_id);
            }
        }

        Ok(())
    }

    /// List all sessions
    pub async fn list_sessions(&self) -> Vec<SessionSummary> {
        let sessions = self.sessions.read().await;
        sessions.values().map(|s| s.to_summary()).collect()
    }

    /// Get session details
    pub async fn get_session(&self, session_id: &str) -> Option<SessionDetails> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| s.to_details())
    }

    /// Get session logs
    pub async fn get_session_logs(
        &self,
        session_id: &str,
        cursor: Option<u64>,
        limit: Option<usize>,
    ) -> Option<LogsResponse> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).map(|s| {
            let mut response = s.runtime.logs.get_logs(cursor, limit.unwrap_or(100));
            response.session_id = session_id.to_string();
            response
        })
    }

    /// Stop all sessions
    pub async fn stop_all(&self) -> Result<()> {
        let session_ids: Vec<String> = {
            let sessions = self.sessions.read().await;
            sessions.keys().cloned().collect()
        };

        for id in session_ids {
            if let Err(e) = self.stop_session(&id).await {
                error!("Failed to stop session {}: {}", id, e);
            }
        }

        self.emit_event("session://all_stopped", serde_json::json!({}));
        info!("Stopped all sessions");
        Ok(())
    }

    /// Refresh stats for all running sessions (throttled 1Hz per session)
    pub async fn refresh_all_stats(&self) {
        let mut sessions = self.sessions.write().await;
        let now_ms = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        
        let mut updated_sessions: Vec<SessionSummary> = Vec::new();
        
        for session in sessions.values_mut() {
            if session.stats.status != SessionStatus::Running {
                continue;
            }

            // Throttle: skip if last emit was < 1s ago
            if now_ms.saturating_sub(session.runtime.last_stats_emit) < STATS_THROTTLE_MS {
                continue;
            }

            // Update uptime
            let now_secs = now_ms / 1000;
            session.stats.uptime_secs = now_secs.saturating_sub(session.runtime.start_time);

            // Get stats from adapter
            match session.config.miner_kind {
                MinerKind::XMRig => {
                    if let Some(adapter) = &session.runtime.xmrig_adapter {
                        if let Ok(stats) = adapter.get_stats().await {
                            session.stats.hashrate_current = stats.current_hashrate();
                            session.stats.hashrate_avg60 = stats.avg_hashrate();
                            session.stats.accepted = stats.accepted_shares();
                            session.stats.rejected = stats.rejected_shares();
                            session.stats.stats_confidence = 1.0;
                            session.stats.telemetry_confidence = TelemetryConfidence::High;
                            session.stats.telemetry_reason = "XMRig HTTP API".to_string();
                            session.stats.connection_state = ConnectionState::Authorized;
                        }
                    }
                }
                MinerKind::CpuminerOpt => {
                    if let Some(adapter) = &session.runtime.cpuminer_adapter {
                        let stats = adapter.get_stats();
                        session.stats.hashrate_current = stats.hashrate;
                        session.stats.hashrate_avg60 = stats.avg_hashrate;
                        session.stats.accepted = stats.accepted;
                        session.stats.rejected = stats.rejected;
                        
                        // Set confidence based on parsed data
                        if stats.hashrate > 0.0 {
                            session.stats.stats_confidence = 0.7;
                            session.stats.telemetry_confidence = TelemetryConfidence::Medium;
                            session.stats.telemetry_reason = "Log parsing".to_string();
                        } else {
                            session.stats.stats_confidence = 0.0;
                            session.stats.telemetry_confidence = TelemetryConfidence::Low;
                            session.stats.telemetry_reason = "No telemetry from miner output".to_string();
                        }
                        
                        // Connection state from shares
                        if stats.accepted > 0 {
                            session.stats.connection_state = ConnectionState::Authorized;
                        } else {
                            session.stats.connection_state = ConnectionState::Connecting;
                        }
                    }
                }
            }
            
            session.runtime.last_stats_emit = now_ms;
            updated_sessions.push(session.to_summary());
        }
        
        // Emit batch update if any sessions updated
        if !updated_sessions.is_empty() {
            if let Some(handle) = &self.app_handle {
                let _ = handle.emit_all("session://batch_updated", serde_json::json!({
                    "sessions": updated_sessions,
                }));
            }
        }
    }

    /// Add log line to session (batched emission)
    pub async fn add_log(&self, session_id: &str, line: String) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.runtime.logs.push(line.clone());
            session.runtime.pending_logs.push(line);
            
            // Batch emit when threshold reached
            if session.runtime.pending_logs.len() >= LOG_BATCH_SIZE {
                let batch = std::mem::take(&mut session.runtime.pending_logs);
                if let Some(handle) = &self.app_handle {
                    let _ = handle.emit_all("session://log_batch", serde_json::json!({
                        "session_id": session_id,
                        "lines": batch,
                    }));
                }
            }
        }
    }

    /// Flush pending logs for a session
    pub async fn flush_logs(&self, session_id: &str) {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            if !session.runtime.pending_logs.is_empty() {
                let batch = std::mem::take(&mut session.runtime.pending_logs);
                if let Some(handle) = &self.app_handle {
                    let _ = handle.emit_all("session://log_batch", serde_json::json!({
                        "session_id": session_id,
                        "lines": batch,
                    }));
                }
            }
        }
    }

    /// Get active session count
    pub async fn active_count(&self) -> usize {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|s| s.stats.status == SessionStatus::Running || s.stats.status == SessionStatus::Suspended)
            .count()
    }

    /// Export sessions for crash recovery (non-secret data only)
    pub async fn export_for_recovery(&self) -> Vec<SessionConfig> {
        let sessions = self.sessions.read().await;
        sessions.values()
            .filter(|s| s.stats.status == SessionStatus::Running)
            .map(|s| s.config.clone())
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_buffer() {
        let mut buffer = LogBuffer::new();
        
        for i in 0..10 {
            buffer.push(format!("Line {}", i));
        }
        
        let response = buffer.get_logs(None, 5);
        assert_eq!(response.lines.len(), 5);
        assert!(response.has_more);
        
        let response = buffer.get_logs(None, 100);
        assert_eq!(response.lines.len(), 10);
        assert!(!response.has_more);
    }

    #[test]
    fn test_session_status_default() {
        let status = SessionStatus::default();
        assert_eq!(status, SessionStatus::Stopped);
    }
}
