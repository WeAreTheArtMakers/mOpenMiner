//! Crash recovery: detect unclean shutdown and offer to resume.
//! IMPORTANT: This does NOT auto-start mining. User must explicitly confirm.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tracing::{info, warn};

const LOCK_FILE_NAME: &str = "mining.lock";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningSession {
    pub coin: String,
    pub pool: String,
    pub wallet: String,
    pub worker: String,
    pub started_at: u64,
    pub pid: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrashRecoveryState {
    pub had_unclean_shutdown: bool,
    pub last_session: Option<MiningSession>,
}

impl Default for CrashRecoveryState {
    fn default() -> Self {
        Self {
            had_unclean_shutdown: false,
            last_session: None,
        }
    }
}

fn lock_file_path() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("openminedash")
        .join(LOCK_FILE_NAME)
}

/// Check for unclean shutdown on app start
pub fn check_crash_recovery() -> CrashRecoveryState {
    let lock_path = lock_file_path();
    
    if lock_path.exists() {
        // Lock file exists = previous session didn't clean up
        match std::fs::read_to_string(&lock_path) {
            Ok(content) => {
                match serde_json::from_str::<MiningSession>(&content) {
                    Ok(session) => {
                        info!("Found unclean shutdown, last session: {:?}", session);
                        // Clean up the lock file
                        let _ = std::fs::remove_file(&lock_path);
                        return CrashRecoveryState {
                            had_unclean_shutdown: true,
                            last_session: Some(session),
                        };
                    }
                    Err(e) => {
                        warn!("Failed to parse lock file: {}", e);
                        let _ = std::fs::remove_file(&lock_path);
                    }
                }
            }
            Err(e) => {
                warn!("Failed to read lock file: {}", e);
                let _ = std::fs::remove_file(&lock_path);
            }
        }
    }
    
    CrashRecoveryState::default()
}

/// Create lock file when mining starts
pub fn create_mining_lock(session: &MiningSession) -> std::io::Result<()> {
    let lock_path = lock_file_path();
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string(session)?;
    std::fs::write(lock_path, content)?;
    info!("Created mining lock file");
    Ok(())
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
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_crash_recovery_flow() {
        // Clean state
        remove_mining_lock();
        let state = check_crash_recovery();
        assert!(!state.had_unclean_shutdown);
        assert!(state.last_session.is_none());

        // Create lock
        let session = MiningSession {
            coin: "xmr".to_string(),
            pool: "pool.example.com:3333".to_string(),
            wallet: "wallet123".to_string(),
            worker: "worker1".to_string(),
            started_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            pid: std::process::id(),
        };
        create_mining_lock(&session).unwrap();

        // Simulate crash (don't call remove_mining_lock)
        let state = check_crash_recovery();
        assert!(state.had_unclean_shutdown);
        assert!(state.last_session.is_some());

        // Lock should be cleaned up after check
        let state = check_crash_recovery();
        assert!(!state.had_unclean_shutdown);
    }
}
