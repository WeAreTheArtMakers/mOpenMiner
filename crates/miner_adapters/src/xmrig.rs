use crate::{AdapterError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::PathBuf;
use std::process::Stdio;
use std::sync::atomic::{AtomicU16, Ordering};
use tauri::Manager;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::{Child, Command};
use tokio::time::{timeout, Duration};
use tracing::{error, info, warn};

/// Base port for XMRig HTTP API - will try incrementing if busy
const XMRIG_API_PORT_BASE: u16 = 45580;
const XMRIG_API_PORT_RANGE: u16 = 20;

/// Current API port (may change if base port is busy)
static CURRENT_API_PORT: AtomicU16 = AtomicU16::new(XMRIG_API_PORT_BASE);

/// Pinned checksums - embedded in binary, not fetched remotely
const PINNED_CHECKSUMS: &str = include_str!("../../../assets/checksums/xmrig.json");

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiningConfig {
    pub coin: String,
    pub pool: String,
    pub wallet: String,
    pub worker: String,
    pub threads: u32,
    pub preset: PerformancePreset,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PerformancePreset {
    Eco,
    #[default]
    Balanced,
    Max,
}

impl PerformancePreset {
    pub fn cpu_priority(&self) -> u8 {
        match self {
            Self::Eco => 1,
            Self::Balanced => 2,
            Self::Max => 5,
        }
    }

    pub fn thread_multiplier(&self) -> f32 {
        match self {
            Self::Eco => 0.25,
            Self::Balanced => 0.5,
            Self::Max => 0.75,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum MinerState {
    Stopped,
    Starting,
    Running,
    Stopping,
    Error,
}

pub struct XMRigAdapter {
    binary_path: Option<PathBuf>,
    custom_binary_path: Option<PathBuf>,
    state: MinerState,
}

impl XMRigAdapter {
    pub fn new() -> Self {
        Self {
            binary_path: None,
            custom_binary_path: None,
            state: MinerState::Stopped,
        }
    }

    pub fn state(&self) -> MinerState {
        self.state
    }

    pub fn set_custom_binary_path(&mut self, path: Option<PathBuf>) {
        self.custom_binary_path = path;
    }

    pub fn binary_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openminedash")
            .join("bin")
    }

    pub fn binary_path() -> PathBuf {
        Self::binary_dir().join("xmrig")
    }

    fn get_pinned_checksum() -> Option<String> {
        let checksums: serde_json::Value = serde_json::from_str(PINNED_CHECKSUMS).ok()?;
        let recommended = checksums["recommended"].as_str()?;
        
        #[cfg(target_arch = "aarch64")]
        let platform = "macos-arm64";
        #[cfg(target_arch = "x86_64")]
        let platform = "macos-x64";
        
        checksums["versions"][recommended][platform]["sha256"]
            .as_str()
            .map(|s| s.to_string())
    }

    pub async fn ensure_binary(&mut self) -> Result<PathBuf> {
        // Prefer custom binary path if set
        if let Some(custom) = &self.custom_binary_path {
            if custom.exists() {
                info!("Using custom binary path: {:?}", custom);
                return Ok(custom.clone());
            }
        }

        let path = Self::binary_path();

        if path.exists() {
            match self.verify_checksum(&path).await {
                Ok(true) => {
                    self.binary_path = Some(path.clone());
                    return Ok(path);
                }
                Ok(false) => {
                    warn!("XMRig checksum mismatch - binary may be tampered");
                    return Err(AdapterError::ChecksumMismatch);
                }
                Err(e) => {
                    warn!("Checksum verification failed: {}", e);
                }
            }
        }

        // Check for quarantine attribute on macOS
        #[cfg(target_os = "macos")]
        if path.exists() {
            if let Ok(output) = std::process::Command::new("xattr")
                .args(["-l", path.to_str().unwrap_or("")])
                .output()
            {
                let attrs = String::from_utf8_lossy(&output.stdout);
                if attrs.contains("com.apple.quarantine") {
                    return Err(AdapterError::Quarantined(
                        "XMRig binary is quarantined by macOS. Go to System Settings → Privacy & Security to allow it, or use Settings → Binary Path to select a manually verified binary.".to_string()
                    ));
                }
            }
        }

        Err(AdapterError::BinaryNotFound(
            "XMRig binary not found. Download from https://github.com/xmrig/xmrig/releases or set a custom path in Settings.".to_string(),
        ))
    }

    async fn verify_checksum(&self, path: &PathBuf) -> Result<bool> {
        let content = tokio::fs::read(path).await?;
        let mut hasher = Sha256::new();
        hasher.update(&content);
        let computed_hash = hex::encode(hasher.finalize());

        let pinned_hash = Self::get_pinned_checksum();
        
        match pinned_hash {
            Some(expected) if !expected.starts_with("REPLACE") => {
                let matches = computed_hash == expected;
                if !matches {
                    error!(
                        "Checksum mismatch! Expected: {}, Got: {}",
                        expected, computed_hash
                    );
                }
                Ok(matches)
            }
            _ => {
                // Development mode - warn but allow
                warn!(
                    "Checksum verification skipped (no pinned hash). Computed: {}",
                    computed_hash
                );
                Ok(true)
            }
        }
    }

    /// Find an available port for HTTP API
    fn find_available_port() -> u16 {
        use std::net::TcpListener;
        
        for offset in 0..XMRIG_API_PORT_RANGE {
            let port = XMRIG_API_PORT_BASE + offset;
            if TcpListener::bind(("127.0.0.1", port)).is_ok() {
                CURRENT_API_PORT.store(port, Ordering::SeqCst);
                return port;
            }
        }
        // Fallback to base port
        XMRIG_API_PORT_BASE
    }

    pub async fn start(&mut self, config: &MiningConfig, app_handle: tauri::AppHandle) -> Result<Child> {
        if self.state == MinerState::Running || self.state == MinerState::Starting {
            return Err(AdapterError::Process("Miner already running or starting".to_string()));
        }

        self.state = MinerState::Starting;
        let binary = match self.ensure_binary().await {
            Ok(b) => b,
            Err(e) => {
                self.state = MinerState::Error;
                return Err(e);
            }
        };

        // Find available port for HTTP API
        let api_port = Self::find_available_port();
        info!("Starting XMRig with pool: {}, preset: {:?}, API port: {}", config.pool, config.preset, api_port);

        // Calculate threads based on preset
        let available_threads = num_cpus::get() as u32;
        let threads = if config.threads > 0 {
            config.threads
        } else {
            ((available_threads as f32) * config.preset.thread_multiplier()).max(1.0) as u32
        };

        let mut cmd = Command::new(&binary);
        cmd.args([
            "-o", &config.pool,
            "-u", &config.wallet,
            "-p", &config.worker,
            "-t", &threads.to_string(),
            "--cpu-priority", &config.preset.cpu_priority().to_string(),
            "--http-enabled",
            "--http-host", "127.0.0.1",
            "--http-port", &api_port.to_string(),
            "--no-color",
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                self.state = MinerState::Error;
                return Err(AdapterError::Process(e.to_string()));
            }
        };

        // Stream stdout to UI
        if let Some(stdout) = child.stdout.take() {
            let handle = app_handle.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stdout);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = handle.emit_all("miner-log", &line);
                }
            });
        }

        // Stream stderr to UI
        if let Some(stderr) = child.stderr.take() {
            let handle = app_handle.clone();
            tokio::spawn(async move {
                let reader = BufReader::new(stderr);
                let mut lines = reader.lines();
                while let Ok(Some(line)) = lines.next_line().await {
                    let _ = handle.emit_all("miner-log", &line);
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
        let api_port = CURRENT_API_PORT.load(Ordering::SeqCst);
        info!("Stopping XMRig (graceful → force sequence)");

        // Step 1: Try graceful shutdown via API
        let client = reqwest::Client::new();
        let _ = client
            .post(format!("http://127.0.0.1:{}/json_rpc", api_port))
            .json(&serde_json::json!({
                "method": "pause",
                "id": 1
            }))
            .timeout(Duration::from_secs(1))
            .send()
            .await;

        // Step 2: SIGTERM
        #[cfg(unix)]
        {
            use nix::sys::signal::{kill, Signal};
            use nix::unistd::Pid;

            if let Some(pid) = child.id() {
                info!("Sending SIGTERM to PID {}", pid);
                let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
            }
        }

        // Step 3: Wait with timeout
        match timeout(Duration::from_secs(3), child.wait()).await {
            Ok(Ok(status)) => {
                info!("XMRig stopped gracefully with status: {}", status);
            }
            Ok(Err(e)) => {
                error!("Error waiting for XMRig: {}", e);
            }
            Err(_) => {
                // Step 4: Force kill
                warn!("XMRig did not stop gracefully after 3s, sending SIGKILL");
                let _ = child.kill().await;
                let _ = child.wait().await;
            }
        }

        self.state = MinerState::Stopped;
    }

    /// Get stats from XMRig HTTP API (preferred over log parsing)
    pub async fn get_stats(&self) -> Result<XMRigStats> {
        if self.state != MinerState::Running {
            return Ok(XMRigStats::default());
        }

        let api_port = CURRENT_API_PORT.load(Ordering::SeqCst);
        let client = reqwest::Client::new();
        let resp = client
            .get(format!("http://127.0.0.1:{}/2/summary", api_port))
            .timeout(Duration::from_secs(2))
            .send()
            .await
            .map_err(|e| AdapterError::Process(e.to_string()))?;

        let stats = resp
            .json()
            .await
            .map_err(|e| AdapterError::Process(e.to_string()))?;

        Ok(stats)
    }
}

impl Default for XMRigAdapter {
    fn default() -> Self {
        Self::new()
    }
}

/// Drop guard ensures process is killed even on panic
impl Drop for XMRigAdapter {
    fn drop(&mut self) {
        if self.state == MinerState::Running {
            warn!("XMRigAdapter dropped while running - process will be killed by kill_on_drop");
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XMRigStats {
    pub hashrate: XMRigHashrate,
    pub results: XMRigResults,
    pub connection: XMRigConnection,
    pub cpu: Option<XMRigCpu>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XMRigHashrate {
    pub total: Vec<Option<f64>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XMRigResults {
    pub shares_good: u64,
    pub shares_total: u64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XMRigConnection {
    pub uptime: u64,
    pub pool: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XMRigCpu {
    pub brand: String,
    pub cores: u32,
    pub threads: u32,
}

impl XMRigStats {
    pub fn current_hashrate(&self) -> f64 {
        self.hashrate.total.first().copied().flatten().unwrap_or(0.0)
    }

    pub fn avg_hashrate(&self) -> f64 {
        self.hashrate.total.get(2).copied().flatten().unwrap_or(0.0)
    }

    pub fn accepted_shares(&self) -> u64 {
        self.results.shares_good
    }

    pub fn rejected_shares(&self) -> u64 {
        self.results.shares_total.saturating_sub(self.results.shares_good)
    }
}
