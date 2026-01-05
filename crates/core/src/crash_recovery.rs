//! Crash recovery: detect unclean shutdown and offer to resume.
//! IMPORTANT: This does NOT auto-start mining. User must explicitly confirm.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

const LOCK_FILE_NAME: &str = "mining.lock";

/// Single mining session info (legacy, for backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningSession {
    pub coin: String,
    pub pool: String,
    pub wallet: String,
    pub worker: String,
    pub started_at: u64,
    pub pid: u32,
}

/// Multi-session snapshot for crash recovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub session_id: String,
    pub coin_id: String,
    pub symbol: String,
    pub algorithm: String,
    pub pool_url: String,
    pub wallet: String,
    pub worker: String,
    pub preset: String,
    pub threads_hint: u32,
    pub status: String,
    pub started_at: u64,
    #[serde(default)]
    pub config_hash: String,
}

/// Lock file content for multi-session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSessionLock {
    pub version: u32,
    pub sessions: Vec<SessionSnapshot>,
    pub saved_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecoveryState {
    pub had_unclean_shutdown: bool,
    pub last_session: Option<MiningSession>,
    pub sessions: Vec<SessionSnapshot>,
}

impl Default for CrashRecoveryState {
    fn default() -> Self {
        Self {
            had_unclean_shutdown: false,
            last_session: None,
            sessions: Vec::new(),
        }
    }
}

fn lock_file_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openminedash")
        .join(LOCK_FILE_NAME)
}

fn temp_lock_file_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openminedash")
        .join(format!("{}.tmp", LOCK_FILE_NAME))
}

/// Check for unclean shutdown on app start
pub fn check_crash_recovery() -> CrashRecoveryState {
    let lock_path = lock_file_path();
    
    // Clean up any stale temp file
    let temp_path = temp_lock_file_path();
    let _ = std::fs::remove_file(&temp_path);
    
    if lock_path.exists() {
        match std::fs::read_to_string(&lock_path) {
            Ok(content) => {
                // Try multi-session format first
                if let Ok(multi) = serde_json::from_str::<MultiSessionLock>(&content) {
                    let _ = std::fs::remove_file(&lock_path);
                    info!("Found unclean shutdown with {} sessions", multi.sessions.len());
                    return CrashRecoveryState {
                        had_unclean_shutdown: true,
                        last_session: None,
                        sessions: multi.sessions,
                    };
                }
                
                // Fallback: legacy format
                if let Ok(session) = serde_json::from_str::<MiningSession>(&content) {
                    let _ = std::fs::remove_file(&lock_path);
                    info!("Found unclean shutdown (legacy): {:?}", session);
                    return CrashRecoveryState {
                        had_unclean_shutdown: true,
                        last_session: Some(session),
                        sessions: Vec::new(),
                    };
                }
                
                // Corrupted lock file - log and remove
                warn!("Corrupted lock file, ignoring: {}", 
                    if content.len() > 100 { &content[..100] } else { &content });
                let _ = std::fs::remove_file(&lock_path);
            }
            Err(e) => {
                warn!("Failed to read lock file: {}", e);
                let _ = std::fs::remove_file(&lock_path);
            }
        }
    }
    
    CrashRecoveryState::default()
}

/// Atomic write: temp file -> fsync -> rename
fn atomic_write(path: &PathBuf, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    
    let temp_path = temp_lock_file_path();
    
    // Ensure parent directory exists for both paths
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    if let Some(parent) = temp_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    
    // Write to temp file
    {
        let mut file = std::fs::File::create(&temp_path)?;
        file.write_all(content.as_bytes())?;
        file.sync_all()?; // fsync
    }
    
    // Atomic rename
    std::fs::rename(&temp_path, path)?;
    
    Ok(())
}

/// Create lock file (legacy single session)
pub fn create_mining_lock(session: &MiningSession) -> std::io::Result<()> {
    let lock_path = lock_file_path();
    let content = serde_json::to_string(session)?;
    atomic_write(&lock_path, &content)?;
    info!("Created mining lock file (atomic)");
    Ok(())
}

/// Create lock file with multi-session snapshot
pub fn create_sessions_lock(sessions: &[SessionSnapshot]) -> std::io::Result<()> {
    let lock_path = lock_file_path();
    
    let content = MultiSessionLock {
        version: 1,
        sessions: sessions.to_vec(),
        saved_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    };
    
    let json = serde_json::to_string(&content)?;
    atomic_write(&lock_path, &json)?;
    info!("Created multi-session lock file (atomic)");
    Ok(())
}

/// Update lock file with current sessions
pub fn update_sessions_lock(sessions: &[SessionSnapshot]) -> std::io::Result<()> {
    if sessions.is_empty() {
        remove_mining_lock();
        return Ok(());
    }
    create_sessions_lock(sessions)
}

/// Remove lock file when mining stops cleanly
pub fn remove_mining_lock() {
    let lock_path = lock_file_path();
    if lock_path.exists() {
        match std::fs::remove_file(&lock_path) {
            Ok(_) => info!("Removed mining lock file"),
            Err(e) => warn!("Failed to remove lock file: {}", e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;
    use std::time::{SystemTime, UNIX_EPOCH};
    
    // Ensure crash recovery tests don't run in parallel
    static TEST_MUTEX: Mutex<()> = Mutex::new(());

    #[test]
    fn test_crash_recovery_flows() {
        let _guard = TEST_MUTEX.lock().unwrap();
        
        // Ensure directory exists
        let lock_path = lock_file_path();
        if let Some(parent) = lock_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        // Clean state
        remove_mining_lock();
        let state = check_crash_recovery();
        assert!(!state.had_unclean_shutdown);

        // Legacy flow
        let session = MiningSession {
            coin: "xmr".to_string(),
            pool: "pool.example.com:3333".to_string(),
            wallet: "wallet123".to_string(),
            worker: "worker1".to_string(),
            started_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            pid: std::process::id(),
        };
        create_mining_lock(&session).unwrap();
        let state = check_crash_recovery();
        assert!(state.had_unclean_shutdown);
        assert!(state.last_session.is_some());

        // Multi-session flow
        let sessions = vec![
            SessionSnapshot {
                session_id: "s1".to_string(),
                coin_id: "xmr".to_string(),
                symbol: "XMR".to_string(),
                algorithm: "randomx".to_string(),
                pool_url: "pool1:3333".to_string(),
                wallet: "w1".to_string(),
                worker: "w1".to_string(),
                preset: "balanced".to_string(),
                threads_hint: 4,
                status: "running".to_string(),
                started_at: 123,
                config_hash: "abc123".to_string(),
            },
        ];
        create_sessions_lock(&sessions).unwrap();
        let state = check_crash_recovery();
        assert!(state.had_unclean_shutdown);
        assert_eq!(state.sessions.len(), 1);
        
        // Clean after check
        let state = check_crash_recovery();
        assert!(!state.had_unclean_shutdown);
    }

    #[test]
    fn test_corrupted_lock_handling() {
        let _guard = TEST_MUTEX.lock().unwrap();
        
        let lock_path = lock_file_path();
        if let Some(parent) = lock_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        
        // Write corrupted content
        std::fs::write(&lock_path, "not valid json {{{").unwrap();
        
        // Should not panic, should return default state
        let state = check_crash_recovery();
        assert!(!state.had_unclean_shutdown);
        
        // Lock file should be removed
        assert!(!lock_path.exists());
    }
}

