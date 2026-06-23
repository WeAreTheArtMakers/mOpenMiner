//! Fake miner adapters for testing without real binaries.
//! Produces deterministic logs and stats for CI/integration tests.

use crate::{AdapterError, CpuminerOptStats, MinerState, MiningConfig, PerformancePreset, Result, XMRigStats};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use tauri::Manager;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration};
use tracing::info;

pub struct FakeMinerAdapter {
    state: MinerState,
    stop_signal: Option<mpsc::Sender<()>>,
    stats: Arc<FakeStats>,
}

struct FakeStats {
    running: AtomicBool,
    hashrate: AtomicU64,
    accepted: AtomicU64,
    rejected: AtomicU64,
    uptime: AtomicU64,
}

impl FakeMinerAdapter {
    pub fn new() -> Self {
        Self {
            state: MinerState::Stopped,
            stop_signal: None,
            stats: Arc::new(FakeStats {
                running: AtomicBool::new(false),
                hashrate: AtomicU64::new(0),
                accepted: AtomicU64::new(0),
                rejected: AtomicU64::new(0),
                uptime: AtomicU64::new(0),
            }),
        }
    }

    pub fn state(&self) -> MinerState {
        self.state
    }

    pub async fn start(&mut self, config: &MiningConfig, app_handle: tauri::AppHandle) -> Result<()> {
        if self.state == MinerState::Running {
            return Err(AdapterError::Process("Already running".to_string()));
        }

        self.state = MinerState::Starting;
        info!("FakeMiner starting with pool: {}", config.pool);

        let (tx, mut rx) = mpsc::channel::<()>(1);
        self.stop_signal = Some(tx);

        let stats = self.stats.clone();
        stats.running.store(true, Ordering::SeqCst);
        stats.hashrate.store(0, Ordering::SeqCst);
        stats.accepted.store(0, Ordering::SeqCst);
        stats.rejected.store(0, Ordering::SeqCst);
        stats.uptime.store(0, Ordering::SeqCst);

        let base_hashrate = match config.preset {
            PerformancePreset::Eco => 500,
            PerformancePreset::Balanced => 1000,
            PerformancePreset::Max => 1500,
        };

        // Spawn fake mining loop
        let handle = app_handle.clone();
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(1));
            let mut second = 0u64;

            // Initial connection logs
            let _ = handle.emit_all("miner-log", "[INFO] XMRig 6.21.0 (fake)");
            let _ = handle.emit_all("miner-log", &format!("[INFO] Connecting to pool..."));

            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        let _ = handle.emit_all("miner-log", "[INFO] Stopping...");
                        stats.running.store(false, Ordering::SeqCst);
                        break;
                    }
                    _ = tick.tick() => {
                        second += 1;
                        stats.uptime.store(second, Ordering::SeqCst);

                        // Simulate hashrate with small variance
                        let variance = (second % 10) as u64 * 10;
                        let hr = base_hashrate + variance;
                        stats.hashrate.store(hr, Ordering::SeqCst);

                        // Accept share every 5 seconds
                        if second % 5 == 0 {
                            let accepted = stats.accepted.fetch_add(1, Ordering::SeqCst) + 1;
                            let _ = handle.emit_all("miner-log", 
                                &format!("[INFO] accepted ({}/0) diff {} ({}ms)", accepted, 100000, 50));
                        }

                        // Reject share every 30 seconds (rare)
                        if second % 30 == 0 && second > 0 {
                            stats.rejected.fetch_add(1, Ordering::SeqCst);
                            let _ = handle.emit_all("miner-log", "[WARN] rejected share");
                        }

                        // Periodic speed log
                        if second % 10 == 0 {
                            let _ = handle.emit_all("miner-log", 
                                &format!("[INFO] speed 10s/60s/15m {:.1} {:.1} {:.1} H/s", 
                                    hr as f64, hr as f64 * 0.98, hr as f64 * 0.95));
                        }
                    }
                }
            }
        });

        self.state = MinerState::Running;
        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.stop_signal.take() {
            let _ = tx.send(()).await;
        }
        self.state = MinerState::Stopped;
    }

    pub fn get_stats(&self) -> XMRigStats {
        let hr = self.stats.hashrate.load(Ordering::SeqCst) as f64;
        XMRigStats {
            hashrate: crate::XMRigHashrate {
                total: vec![Some(hr), Some(hr * 0.98), Some(hr * 0.95)],
            },
            results: crate::XMRigResults {
                shares_good: self.stats.accepted.load(Ordering::SeqCst),
                shares_total: self.stats.accepted.load(Ordering::SeqCst) 
                    + self.stats.rejected.load(Ordering::SeqCst),
            },
            connection: crate::XMRigConnection {
                uptime: self.stats.uptime.load(Ordering::SeqCst),
                pool: "fake-pool.example.com:3333".to_string(),
            },
            cpu: None,
        }
    }
}

impl Default for FakeMinerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_fake_miner_state_machine() {
        let mut adapter = FakeMinerAdapter::new();
        assert_eq!(adapter.state(), MinerState::Stopped);

        // Can't test start without tauri app handle in unit tests
        // Integration tests would use a mock app handle
    }

    #[test]
    fn test_fake_stats_default() {
        let adapter = FakeMinerAdapter::new();
        let stats = adapter.get_stats();
        assert_eq!(stats.results.shares_good, 0);
        assert_eq!(stats.connection.uptime, 0);
    }
}


// ============================================================================
// FakeCpuminerAdapter - For testing cpuminer-opt routing without real binary
// ============================================================================

pub struct FakeCpuminerAdapter {
    state: MinerState,
    stop_signal: Option<mpsc::Sender<()>>,
    stats: Arc<FakeCpuminerStats>,
    algorithm: String,
}

struct FakeCpuminerStats {
    running: AtomicBool,
    hashrate: AtomicU64,
    accepted: AtomicU64,
    rejected: AtomicU64,
    uptime: AtomicU64,
}

impl FakeCpuminerAdapter {
    pub fn new() -> Self {
        Self {
            state: MinerState::Stopped,
            stop_signal: None,
            stats: Arc::new(FakeCpuminerStats {
                running: AtomicBool::new(false),
                hashrate: AtomicU64::new(0),
                accepted: AtomicU64::new(0),
                rejected: AtomicU64::new(0),
                uptime: AtomicU64::new(0),
            }),
            algorithm: String::new(),
        }
    }

    pub fn state(&self) -> MinerState {
        self.state
    }

    pub async fn start(&mut self, config: &MiningConfig, app_handle: tauri::AppHandle) -> Result<()> {
        if self.state == MinerState::Running {
            return Err(AdapterError::Process("Already running".to_string()));
        }

        self.state = MinerState::Starting;
        self.algorithm = config.coin.clone();
        info!("FakeCpuminer starting with algo: {}, pool: {}", config.coin, config.pool);

        let (tx, mut rx) = mpsc::channel::<()>(1);
        self.stop_signal = Some(tx);

        let stats = self.stats.clone();
        stats.running.store(true, Ordering::SeqCst);
        stats.hashrate.store(0, Ordering::SeqCst);
        stats.accepted.store(0, Ordering::SeqCst);
        stats.rejected.store(0, Ordering::SeqCst);
        stats.uptime.store(0, Ordering::SeqCst);

        // cpuminer-opt has much lower hashrates for SHA256/Scrypt on CPU
        let base_hashrate = match config.preset {
            PerformancePreset::Eco => 50,      // ~50 H/s for SHA256d on CPU
            PerformancePreset::Balanced => 100,
            PerformancePreset::Max => 150,
        };

        let algo = config.coin.clone();
        let handle = app_handle.clone();
        
        tokio::spawn(async move {
            let mut tick = interval(Duration::from_secs(1));
            let mut second = 0u64;

            // cpuminer-opt style logs
            let _ = handle.emit_all("miner-log", &format!("[INFO] cpuminer-opt 3.24.5 (fake)"));
            let _ = handle.emit_all("miner-log", &format!("[INFO] Using algorithm: {}", algo));
            let _ = handle.emit_all("miner-log", "[INFO] Connecting to stratum server...");

            loop {
                tokio::select! {
                    _ = rx.recv() => {
                        let _ = handle.emit_all("miner-log", "[INFO] Exiting...");
                        stats.running.store(false, Ordering::SeqCst);
                        break;
                    }
                    _ = tick.tick() => {
                        second += 1;
                        stats.uptime.store(second, Ordering::SeqCst);

                        // Simulate very low hashrate with variance
                        let variance = (second % 5) as u64 * 5;
                        let hr = base_hashrate + variance;
                        stats.hashrate.store(hr, Ordering::SeqCst);

                        // Accept share every 10 seconds (slower than XMRig due to difficulty)
                        if second % 10 == 0 {
                            let accepted = stats.accepted.fetch_add(1, Ordering::SeqCst) + 1;
                            let total = accepted + stats.rejected.load(Ordering::SeqCst);
                            let _ = handle.emit_all("miner-log", 
                                &format!("[INFO] accepted: {}/{} (diff {})", accepted, total, 1));
                        }

                        // Reject share every 60 seconds
                        if second % 60 == 0 && second > 0 {
                            stats.rejected.fetch_add(1, Ordering::SeqCst);
                            let _ = handle.emit_all("miner-log", "[WARN] rejected share (stale)");
                        }

                        // Periodic hashrate log (cpuminer style)
                        if second % 15 == 0 {
                            let _ = handle.emit_all("miner-log", 
                                &format!("[INFO] CPU: {:.2} H/s", hr as f64));
                        }
                    }
                }
            }
        });

        self.state = MinerState::Running;
        Ok(())
    }

    pub async fn stop(&mut self) {
        if let Some(tx) = self.stop_signal.take() {
            let _ = tx.send(()).await;
        }
        self.state = MinerState::Stopped;
    }

    pub fn get_stats(&self) -> CpuminerOptStats {
        let hr = self.stats.hashrate.load(Ordering::SeqCst) as f64;
        CpuminerOptStats {
            hashrate: hr,
            avg_hashrate: hr,
            accepted: self.stats.accepted.load(Ordering::SeqCst),
            rejected: self.stats.rejected.load(Ordering::SeqCst),
            difficulty: 1.0,
            uptime: self.stats.uptime.load(Ordering::SeqCst),
            hashrate_unknown: false,
        }
    }
}

impl Default for FakeCpuminerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod fake_cpuminer_tests {
    use super::*;

    #[test]
    fn test_fake_cpuminer_state_machine() {
        let adapter = FakeCpuminerAdapter::new();
        assert_eq!(adapter.state(), MinerState::Stopped);
    }

    #[test]
    fn test_fake_cpuminer_stats_default() {
        let adapter = FakeCpuminerAdapter::new();
        let stats = adapter.get_stats();
        assert_eq!(stats.accepted, 0);
        assert_eq!(stats.uptime, 0);
        assert_eq!(stats.hashrate, 0.0);
    }
}
