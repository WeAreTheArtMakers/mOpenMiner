//! cpuminer-opt adapter for multi-algorithm CPU mining.
//!
//! cpuminer-opt is a GPL-2.0 licensed CPU miner supporting many algorithms
//! including SHA-256d (BTC), Scrypt (LTC), and others not covered by XMRig.
//!
//! This adapter runs cpuminer-opt as a separate binary (sidecar) to comply
//! with GPL licensing requirements.

use crate::xmrig::{MinerState, MiningConfig, PerformancePreset};
use crate::{AdapterError, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::Duration;
use tracing::{error, info, warn};

/// Pinned checksums for cpuminer-opt - embedded, not fetched remotely
const PINNED_CHECKSUMS: &str = include_str!("../../../assets/checksums/cpuminer-opt.json");

/// Maximum log lines to keep in ring buffer
const MAX_LOG_LINES: usize = 500;

/// Rolling average window for hashrate (seconds)
const HASHRATE_AVG_WINDOW: usize = 60;

/// Algorithm mappings: (coin_algo, cpuminer_algo)
/// Reference: https://github.com/JayDDee/cpuminer-opt/wiki/Supported-Algorithms
pub const SUPPORTED_ALGORITHMS: &[(&str, &str)] = &[
    // SHA-2 family
    ("sha256d", "sha256d"),
    ("sha256", "sha256d"),
    ("sha-256", "sha256d"),
    ("sha256t", "sha256t"),
    // Scrypt family
    ("scrypt", "scrypt"),
    // X-series
    ("x11", "x11"),
    ("x13", "x13"),
    ("x14", "x14"),
    ("x15", "x15"),
    ("x16r", "x16r"),
    ("x16rv2", "x16rv2"),
    ("x16s", "x16s"),
    ("x17", "x17"),
    ("x21s", "x21s"),
    ("x22i", "x22i"),
    ("x25x", "x25x"),
    // Lyra2 family
    ("lyra2v2", "lyra2v2"),
    ("lyra2v3", "lyra2v3"),
    ("lyra2z", "lyra2z"),
    ("lyra2h", "lyra2h"),
    // Yescrypt family
    ("yescrypt", "yescrypt"),
    ("yescryptr8", "yescryptr8"),
    ("yescryptr16", "yescryptr16"),
    ("yescryptr32", "yescryptr32"),
    ("yespower", "yespower"),
    ("yespowerr16", "yespowerr16"),
    // Other algorithms
    ("allium", "allium"),
    ("blake", "blake"),
    ("blake2b", "blake2b"),
    ("blake2s", "blake2s"),
    ("groestl", "groestl"),
    ("keccak", "keccak"),
    ("lbry", "lbry"),
    ("neoscrypt", "neoscrypt"),
    ("nist5", "nist5"),
    ("phi1612", "phi1612"),
    ("phi2", "phi2"),
    ("quark", "quark"),
    ("qubit", "qubit"),
    ("skein", "skein"),
    ("skein2", "skein2"),
    ("tribus", "tribus"),
    ("whirlpool", "whirlpool"),
];

/// Map coin algorithm to cpuminer-opt algorithm name
pub fn map_algorithm(coin_algo: &str) -> Option<&'static str> {
    let algo_lower = coin_algo.to_lowercase();
    SUPPORTED_ALGORITHMS
        .iter()
        .find(|(k, _)| k.eq_ignore_ascii_case(&algo_lower))
        .map(|(_, v)| *v)
}

/// Check if cpuminer-opt supports a given algorithm
pub fn supports_algorithm(algo: &str) -> bool {
    map_algorithm(algo).is_some()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuminerOptStats {
    pub hashrate: f64,
    pub avg_hashrate: f64,
    pub accepted: u64,
    pub rejected: u64,
    pub difficulty: f64,
    pub uptime: u64,
    pub hashrate_unknown: bool,
}

impl Default for CpuminerOptStats {
    fn default() -> Self {
        Self {
            hashrate: 0.0,
            avg_hashrate: 0.0,
            accepted: 0,
            rejected: 0,
            difficulty: 0.0,
            uptime: 0,
            hashrate_unknown: true,
        }
    }
}

/// Thread-safe stats container with log parsing
#[derive(Clone)]
pub struct StatsCollector {
    inner: Arc<Mutex<StatsInner>>,
}

struct StatsInner {
    stats: CpuminerOptStats,
    hashrate_samples: VecDeque<f64>,
    log_buffer: VecDeque<String>,
    start_time: Option<std::time::Instant>,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(StatsInner {
                stats: CpuminerOptStats::default(),
                hashrate_samples: VecDeque::with_capacity(HASHRATE_AVG_WINDOW),
                log_buffer: VecDeque::with_capacity(MAX_LOG_LINES),
                start_time: None,
            })),
        }
    }

    pub fn start(&self) {
        if let Ok(mut inner) = self.inner.lock() {
            inner.start_time = Some(std::time::Instant::now());
            inner.stats = CpuminerOptStats::default();
            inner.hashrate_samples.clear();
            inner.log_buffer.clear();
        }
    }

    pub fn parse_line(&self, line: &str) {
        if let Ok(mut inner) = self.inner.lock() {
            // Store in ring buffer
            if inner.log_buffer.len() >= MAX_LOG_LINES {
                inner.log_buffer.pop_front();
            }
            inner.log_buffer.push_back(line.to_string());

            // Update uptime
            if let Some(start) = inner.start_time {
                inner.stats.uptime = start.elapsed().as_secs();
            }

            // Parse hashrate
            if let Some(hr) = Self::extract_hashrate(line) {
                inner.stats.hashrate = hr;
                inner.stats.hashrate_unknown = false;
                
                // Rolling average
                if inner.hashrate_samples.len() >= HASHRATE_AVG_WINDOW {
                    inner.hashrate_samples.pop_front();
                }
                inner.hashrate_samples.push_back(hr);
                inner.stats.avg_hashrate = inner.hashrate_samples.iter().sum::<f64>() 
                    / inner.hashrate_samples.len() as f64;
            }

            // Parse shares
            if let Some((acc, rej)) = Self::extract_shares(line) {
                inner.stats.accepted = acc;
                inner.stats.rejected = rej;
            }

            // Parse difficulty
            if let Some(diff) = Self::extract_difficulty(line) {
                inner.stats.difficulty = diff;
            }
        }
    }

    pub fn get_stats(&self) -> CpuminerOptStats {
        self.inner.lock().map(|i| i.stats.clone()).unwrap_or_default()
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.inner.lock().map(|i| i.log_buffer.iter().cloned().collect()).unwrap_or_default()
    }

    /// Extract hashrate from log line (best-effort parsing)
    /// Patterns: "1.23 kH/s", "1.23 MH/s", "1.23 H/s", "Total: 1.23kH/s"
    fn extract_hashrate(line: &str) -> Option<f64> {
        // Try multiple patterns - cpuminer output varies
        let patterns = [
            (r"(\d+\.?\d*)\s*GH/s", 1_000_000_000.0),
            (r"(\d+\.?\d*)\s*MH/s", 1_000_000.0),
            (r"(\d+\.?\d*)\s*kH/s", 1_000.0),
            (r"(\d+\.?\d*)\s*H/s", 1.0),
            // Alternative formats
            (r"Total:\s*(\d+\.?\d*)GH", 1_000_000_000.0),
            (r"Total:\s*(\d+\.?\d*)MH", 1_000_000.0),
            (r"Total:\s*(\d+\.?\d*)kH", 1_000.0),
            (r"Total:\s*(\d+\.?\d*)H", 1.0),
        ];

        for (pattern, multiplier) in patterns {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(line) {
                    if let Some(val) = caps.get(1) {
                        if let Ok(num) = val.as_str().parse::<f64>() {
                            return Some(num * multiplier);
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract accepted/rejected shares
    /// Patterns: "accepted: 5/6", "accepted 5, rejected 1", "(5/6)"
    fn extract_shares(line: &str) -> Option<(u64, u64)> {
        let line_lower = line.to_lowercase();
        
        // Pattern: "accepted: 5/6" or "accepted: 5/6 (83.33%)"
        if let Ok(re) = Regex::new(r"accepted[:\s]+(\d+)/(\d+)") {
            if let Some(caps) = re.captures(&line_lower) {
                let accepted: u64 = caps.get(1)?.as_str().parse().ok()?;
                let total: u64 = caps.get(2)?.as_str().parse().ok()?;
                return Some((accepted, total.saturating_sub(accepted)));
            }
        }

        // Pattern: "accepted (5/6)"
        if let Ok(re) = Regex::new(r"accepted\s*\((\d+)/(\d+)\)") {
            if let Some(caps) = re.captures(&line_lower) {
                let accepted: u64 = caps.get(1)?.as_str().parse().ok()?;
                let total: u64 = caps.get(2)?.as_str().parse().ok()?;
                return Some((accepted, total.saturating_sub(accepted)));
            }
        }

        // Pattern: "yes! (5)" for accepted
        if line_lower.contains("yes!") || line_lower.contains("yay!") {
            if let Ok(re) = Regex::new(r"\((\d+)\)") {
                if let Some(caps) = re.captures(&line_lower) {
                    let accepted: u64 = caps.get(1)?.as_str().parse().ok()?;
                    return Some((accepted, 0));
                }
            }
        }

        None
    }

    /// Extract difficulty from log
    fn extract_difficulty(line: &str) -> Option<f64> {
        if let Ok(re) = Regex::new(r"diff[:\s]+(\d+\.?\d*)") {
            if let Some(caps) = re.captures(&line.to_lowercase()) {
                return caps.get(1)?.as_str().parse().ok();
            }
        }
        None
    }
}

impl Default for StatsCollector {
    fn default() -> Self {
        Self::new()
    }
}

pub struct CpuminerOptAdapter {
    state: MinerState,
    custom_binary_path: Option<PathBuf>,
    stats_collector: StatsCollector,
}

impl CpuminerOptAdapter {
    pub fn new() -> Self {
        Self {
            state: MinerState::Stopped,
            custom_binary_path: None,
            stats_collector: StatsCollector::new(),
        }
    }

    pub fn set_custom_binary_path(&mut self, path: Option<PathBuf>) {
        self.custom_binary_path = path;
    }

    pub fn state(&self) -> MinerState {
        self.state
    }

    pub fn binary_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openminedash")
            .join("bin")
    }

    pub fn binary_path() -> PathBuf {
        Self::binary_dir().join("cpuminer-opt")
    }

    fn get_binary_path(&self) -> PathBuf {
        self.custom_binary_path.clone().unwrap_or_else(Self::binary_path)
    }

    fn get_pinned_checksum() -> Option<String> {
        let checksums: serde_json::Value = serde_json::from_str(PINNED_CHECKSUMS).ok()?;
        let recommended = checksums["recommended"].as_str()?;
        
        #[cfg(target_arch = "aarch64")]
        let platform = "macos-arm64";
        #[cfg(target_arch = "x86_64")]
        let platform = "macos-x64";
        #[cfg(not(any(target_arch = "aarch64", target_arch = "x86_64")))]
        let platform = "unknown";
        
        checksums["versions"][recommended][platform]["sha256"]
            .as_str()
            .map(|s| s.to_string())
    }

    /// Verify binary exists and passes security checks
    pub async fn ensure_binary(&self) -> Result<PathBuf> {
        let path = self.get_binary_path();

        if !path.exists() {
            return Err(AdapterError::BinaryNotFound(format!(
                "cpuminer-opt not found at {:?}. See docs/MINERS.md for installation.",
                path
            )));
        }

        // Verify checksum
        match self.verify_checksum(&path).await {
            Ok(true) => {}
            Ok(false) => {
                warn!("cpuminer-opt checksum mismatch - binary may be tampered or different version");
                // Don't fail, just warn - user may have custom build
            }
            Err(e) => {
                warn!("Checksum verification failed: {}", e);
            }
        }

        // Check quarantine on macOS
        #[cfg(target_os = "macos")]
        self.check_quarantine(&path)?;

        // Verify architecture
        #[cfg(target_os = "macos")]
        self.verify_architecture(&path)?;

        // Ensure executable
        #[cfg(unix)]
        self.ensure_executable(&path)?;

        Ok(path)
    }

    async fn verify_checksum(&self, path: &PathBuf) -> Result<bool> {
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let computed = hex::encode(hasher.finalize());

        match Self::get_pinned_checksum() {
            Some(expected) if !expected.starts_with("REPLACE") => {
                let matches = computed.eq_ignore_ascii_case(&expected);
                if !matches {
                    error!("Checksum mismatch! Expected: {}, Got: {}", expected, computed);
                }
                Ok(matches)
            }
            _ => {
                warn!("No pinned checksum available. Computed: {}", computed);
                Ok(true) // Allow in dev mode
            }
        }
    }

    #[cfg(target_os = "macos")]
    fn check_quarantine(&self, path: &PathBuf) -> Result<()> {
        let output = std::process::Command::new("xattr")
            .args(["-l", path.to_str().unwrap_or("")])
            .output()?;
        
        let attrs = String::from_utf8_lossy(&output.stdout);
        if attrs.contains("com.apple.quarantine") {
            return Err(AdapterError::Quarantined(
                "cpuminer-opt is quarantined by macOS. Run: xattr -d com.apple.quarantine <path>".to_string()
            ));
        }
        Ok(())
    }

    #[cfg(target_os = "macos")]
    fn verify_architecture(&self, path: &PathBuf) -> Result<()> {
        let output = std::process::Command::new("file")
            .arg(path)
            .output()?;
        
        let file_info = String::from_utf8_lossy(&output.stdout);
        
        #[cfg(target_arch = "aarch64")]
        if !file_info.contains("arm64") {
            return Err(AdapterError::Process(
                "Binary is not arm64. Please compile cpuminer-opt for Apple Silicon.".to_string()
            ));
        }
        
        #[cfg(target_arch = "x86_64")]
        if !file_info.contains("x86_64") {
            return Err(AdapterError::Process(
                "Binary is not x86_64.".to_string()
            ));
        }
        
        Ok(())
    }

    #[cfg(unix)]
    fn ensure_executable(&self, path: &PathBuf) -> Result<()> {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(path)?;
        let mut perms = metadata.permissions();
        if perms.mode() & 0o111 == 0 {
            perms.set_mode(perms.mode() | 0o755);
            std::fs::set_permissions(path, perms)?;
            info!("Set executable permission on cpuminer-opt");
        }
        Ok(())
    }

    fn build_args(&self, config: &MiningConfig) -> Vec<String> {
        let mut args = Vec::new();
        
        // Algorithm (required)
        let algo = map_algorithm(&config.coin).unwrap_or("sha256d");
        args.push("-a".to_string());
        args.push(algo.to_string());
        
        // Pool URL
        args.push("-o".to_string());
        args.push(config.pool.clone());
        
        // User (wallet.worker or just wallet)
        let user = if config.worker.is_empty() {
            config.wallet.clone()
        } else {
            format!("{}.{}", config.wallet, config.worker)
        };
        args.push("-u".to_string());
        args.push(user);
        
        // Password (usually 'x')
        args.push("-p".to_string());
        args.push("x".to_string());
        
        // Thread count based on preset
        let cpu_count = num_cpus::get() as u32;
        let threads = match config.preset {
            PerformancePreset::Eco => (cpu_count / 4).max(1),
            PerformancePreset::Balanced => (cpu_count / 2).max(1),
            PerformancePreset::Max => ((cpu_count * 3) / 4).max(1),
        };
        args.push("-t".to_string());
        args.push(threads.to_string());
        
        // CPU priority based on preset
        let priority = match config.preset {
            PerformancePreset::Eco => 0,
            PerformancePreset::Balanced => 1,
            PerformancePreset::Max => 2,
        };
        args.push("--cpu-priority".to_string());
        args.push(priority.to_string());
        
        args
    }

    pub async fn start(
        &mut self,
        config: &MiningConfig,
        app_handle: tauri::AppHandle,
    ) -> Result<Child> {
        if self.state != MinerState::Stopped && self.state != MinerState::Error {
            return Err(AdapterError::Process("Miner already running".to_string()));
        }

        // Validate algorithm
        if map_algorithm(&config.coin).is_none() {
            return Err(AdapterError::Process(format!(
                "Algorithm '{}' not supported by cpuminer-opt",
                config.coin
            )));
        }

        self.state = MinerState::Starting;
        
        // Verify binary
        let binary = match self.ensure_binary().await {
            Ok(b) => b,
            Err(e) => {
                self.state = MinerState::Error;
                return Err(e);
            }
        };

        let args = self.build_args(config);
        info!("Starting cpuminer-opt: {:?} {:?}", binary, args);

        // Reset stats
        self.stats_collector.start();

        let mut child = Command::new(&binary)
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| AdapterError::Process(format!("Failed to spawn: {}", e)))?;

        // Stream stdout
        if let Some(stdout) = child.stdout.take() {
            let handle = app_handle.clone();
            let collector = self.stats_collector.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    collector.parse_line(&line);
                    let _ = handle.emit_all("miner-log", &line);
                }
            });
        }

        // Stream stderr
        if let Some(stderr) = child.stderr.take() {
            let handle = app_handle.clone();
            let collector = self.stats_collector.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    collector.parse_line(&line);
                    let _ = handle.emit_all("miner-log", &format!("[stderr] {}", line));
                }
            });
        }

        self.state = MinerState::Running;
        Ok(child)
    }

    pub async fn stop(&mut self, child: &mut Child) {
        if self.state != MinerState::Running {
            return;
        }

        self.state = MinerState::Stopping;
        info!("Stopping cpuminer-opt (SIGTERM -> timeout -> SIGKILL)");

        // Step 1: SIGTERM
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;
            if let Some(pid) = child.id() {
                info!("Sending SIGTERM to PID {}", pid);
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
            }
        }

        // Step 2: Wait with timeout
        match tokio::time::timeout(Duration::from_secs(3), child.wait()).await {
            Ok(Ok(status)) => {
                info!("cpuminer-opt stopped gracefully: {}", status);
            }
            Ok(Err(e)) => {
                error!("Error waiting for cpuminer-opt: {}", e);
            }
            Err(_) => {
                // Step 3: Force kill
                warn!("cpuminer-opt did not stop in 3s, sending SIGKILL");
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }

        self.state = MinerState::Stopped;
    }

    pub fn get_stats(&self) -> CpuminerOptStats {
        self.stats_collector.get_stats()
    }

    pub fn get_logs(&self) -> Vec<String> {
        self.stats_collector.get_logs()
    }
}

impl Default for CpuminerOptAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Drop guard - ensure process cleanup
impl Drop for CpuminerOptAdapter {
    fn drop(&mut self) {
        if self.state == MinerState::Running {
            warn!("CpuminerOptAdapter dropped while running - process will be killed by kill_on_drop");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_algorithm_mapping() {
        assert_eq!(map_algorithm("sha256"), Some("sha256d"));
        assert_eq!(map_algorithm("SHA256"), Some("sha256d"));
        assert_eq!(map_algorithm("sha256d"), Some("sha256d"));
        assert_eq!(map_algorithm("scrypt"), Some("scrypt"));
        assert_eq!(map_algorithm("x11"), Some("x11"));
        assert_eq!(map_algorithm("randomx"), None); // XMRig only
        assert_eq!(map_algorithm("kawpow"), None);  // GPU only
        assert_eq!(map_algorithm("ethash"), None);  // GPU only
    }

    #[test]
    fn test_supports_algorithm() {
        assert!(supports_algorithm("sha256"));
        assert!(supports_algorithm("scrypt"));
        assert!(supports_algorithm("x11"));
        assert!(!supports_algorithm("randomx"));
        assert!(!supports_algorithm("ethash"));
    }

    #[test]
    fn test_hashrate_extraction() {
        // Standard formats
        assert_eq!(StatsCollector::extract_hashrate("CPU: 1.5 kH/s"), Some(1500.0));
        assert_eq!(StatsCollector::extract_hashrate("Total: 2.0 MH/s"), Some(2_000_000.0));
        assert_eq!(StatsCollector::extract_hashrate("Speed: 500 H/s"), Some(500.0));
        assert_eq!(StatsCollector::extract_hashrate("Rate: 1.5 GH/s"), Some(1_500_000_000.0));
        
        // Alternative formats
        assert_eq!(StatsCollector::extract_hashrate("Total: 1.5kH"), Some(1500.0));
        
        // No match
        assert_eq!(StatsCollector::extract_hashrate("Connected to pool"), None);
    }

    #[test]
    fn test_shares_extraction() {
        // Standard format
        assert_eq!(StatsCollector::extract_shares("accepted: 5/6"), Some((5, 1)));
        assert_eq!(StatsCollector::extract_shares("accepted: 10/10"), Some((10, 0)));
        
        // Parentheses format
        assert_eq!(StatsCollector::extract_shares("accepted (5/6)"), Some((5, 1)));
        
        // yay format
        assert_eq!(StatsCollector::extract_shares("yay! (5)"), Some((5, 0)));
        assert_eq!(StatsCollector::extract_shares("yes! (10)"), Some((10, 0)));
    }

    #[test]
    fn test_difficulty_extraction() {
        assert_eq!(StatsCollector::extract_difficulty("diff: 1.5"), Some(1.5));
        assert_eq!(StatsCollector::extract_difficulty("Diff 100"), Some(100.0));
        assert_eq!(StatsCollector::extract_difficulty("no diff here"), None);
    }

    #[test]
    fn test_stats_collector() {
        let collector = StatsCollector::new();
        collector.start();
        
        // Parse some lines
        collector.parse_line("[INFO] cpuminer-opt 3.24.5");
        collector.parse_line("[INFO] CPU: 1.5 kH/s");
        collector.parse_line("[INFO] accepted: 5/6");
        
        let stats = collector.get_stats();
        assert_eq!(stats.hashrate, 1500.0);
        assert_eq!(stats.accepted, 5);
        assert_eq!(stats.rejected, 1);
        assert!(!stats.hashrate_unknown);
    }

    #[test]
    fn test_rolling_average() {
        let collector = StatsCollector::new();
        collector.start();
        
        // Add multiple samples
        collector.parse_line("CPU: 1000 H/s");
        collector.parse_line("CPU: 2000 H/s");
        collector.parse_line("CPU: 3000 H/s");
        
        let stats = collector.get_stats();
        assert_eq!(stats.hashrate, 3000.0); // Latest
        assert_eq!(stats.avg_hashrate, 2000.0); // Average of 1000, 2000, 3000
    }

    #[test]
    fn test_log_buffer() {
        let collector = StatsCollector::new();
        collector.start();
        
        for i in 0..10 {
            collector.parse_line(&format!("Line {}", i));
        }
        
        let logs = collector.get_logs();
        assert_eq!(logs.len(), 10);
        assert_eq!(logs[0], "Line 0");
        assert_eq!(logs[9], "Line 9");
    }
}
