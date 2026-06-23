mod algo_routing;
mod alert_store;
mod benchmark;
mod config;
mod crash_recovery;
mod diagnostics;
mod mining_history;
mod plugin;
mod pool_audit;
mod process;
mod remote;
mod session_manager;
mod telemetry;
mod thread_budget;

pub use algo_routing::*;
pub use alert_store::*;
pub use benchmark::*;
pub use config::*;
pub use crash_recovery::*;
pub use diagnostics::*;
pub use mining_history::*;
pub use plugin::*;
pub use pool_audit::*;
pub use process::*;
pub use remote::*;
pub use session_manager::*;
pub use telemetry::*;
pub use thread_budget::*;

use openminedash_miner_adapters::{
    CpuminerOptAdapter, MinerState, MiningConfig as AdapterMiningConfig, PerformancePreset,
    XMRigAdapter,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::process::Child;

#[derive(Error, Debug)]
pub enum CoreError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Mining not started")]
    NotRunning,
    #[error("Mining already running")]
    AlreadyRunning,
    #[error("Consent not granted")]
    NoConsent,
    #[error("Miner error: {0}")]
    Miner(String),
    #[error("Invalid state transition")]
    InvalidState,
    #[error("Plugin validation failed: {0}")]
    PluginValidation(String),
}

pub type Result<T> = std::result::Result<T, CoreError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MiningConfig {
    pub coin: String,
    pub pool: String,
    pub wallet: String,
    pub worker: String,
    pub threads: u32,
    #[serde(default)]
    pub preset: PerformancePreset,
    /// Algorithm for the coin (used for routing)
    #[serde(default)]
    pub algorithm: String,
    /// Enable "Try Mining Anyway" mode for non-CPU-optimized coins
    #[serde(default)]
    pub try_anyway: bool,
}

/// Which miner is currently active
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ActiveMiner {
    None,
    XMRig,
    CpuminerOpt,
}

impl From<MiningConfig> for AdapterMiningConfig {
    fn from(c: MiningConfig) -> Self {
        let algo = if c.algorithm.is_empty() { c.coin.clone() } else { c.algorithm.clone() };
        tracing::info!("MiningConfig conversion: coin={}, algorithm={}, using={}", c.coin, c.algorithm, algo);
        AdapterMiningConfig {
            coin: algo,
            pool: c.pool,
            wallet: c.wallet,
            worker: c.worker,
            threads: c.threads,
            preset: c.preset,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningStatus {
    pub state: String,
    pub is_running: bool,
    pub coin: Option<String>,
    pub pool: Option<String>,
    pub wallet: Option<String>,
    pub worker: Option<String>,
    pub hashrate: f64,
    pub avg_hashrate: f64,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub uptime: u64,
    /// Which miner is currently active
    #[serde(default)]
    pub active_miner: String,
    /// Warning message for non-practical mining
    #[serde(default)]
    pub warning: Option<String>,
    /// Timestamp when mining started (for elapsed time calculation)
    #[serde(default)]
    pub started_at: u64,
    /// Algorithm being mined
    #[serde(default)]
    pub algorithm: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub coin: String,
    pub pool: String,
    pub wallet: String,
    pub worker: String,
    pub threads: u32,
    #[serde(default)]
    pub preset: PerformancePreset,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoinDefinition {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    pub id: String,
    pub name: String,
    pub symbol: String,
    pub algorithm: String,
    pub recommended_miner: String,
    pub cpu_mineable: bool,
    pub default_pools: Vec<PoolConfig>,
    pub notes: Option<String>,
    #[serde(default)]
    pub trusted: bool,
}

fn default_schema_version() -> u32 { 1 }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolConfig {
    pub name: String,
    pub stratum_url: String,
    pub tls: bool,
    pub region: String,
}

pub struct AppState {
    config: AppConfig,
    status: MiningStatus,
    miner_process: Option<Child>,
    xmrig_adapter: XMRigAdapter,
    cpuminer_adapter: CpuminerOptAdapter,
    active_miner: ActiveMiner,
    crash_recovery: CrashRecoveryState,
    remote_endpoints: Vec<RemoteEndpoint>,
    mining_history: MiningHistory,
}

impl AppState {
    pub fn new() -> Self {
        let config = AppConfig::load().unwrap_or_default();
        let crash_recovery = check_crash_recovery();
        let mining_history = MiningHistory::load();
        
        Self {
            config,
            status: MiningStatus {
                state: "stopped".to_string(),
                ..Default::default()
            },
            miner_process: None,
            xmrig_adapter: XMRigAdapter::new(),
            cpuminer_adapter: CpuminerOptAdapter::new(),
            active_miner: ActiveMiner::None,
            crash_recovery,
            remote_endpoints: Vec::new(),
            mining_history,
        }
    }

    pub fn has_consent(&self) -> bool {
        self.config.consent
    }

    pub fn set_consent(&mut self, consent: bool) {
        self.config.consent = consent;
    }

    pub fn theme(&self) -> &str {
        &self.config.theme
    }

    pub fn status(&self) -> &MiningStatus {
        &self.status
    }

    pub fn miner_state(&self) -> MinerState {
        self.xmrig_adapter.state()
    }

    pub fn crash_recovery_state(&self) -> &CrashRecoveryState {
        &self.crash_recovery
    }

    pub fn clear_crash_recovery(&mut self) {
        self.crash_recovery = CrashRecoveryState::default();
    }

    pub fn profiles(&self) -> &[Profile] {
        &self.config.profiles
    }

    pub fn custom_binary_path(&self) -> Option<&PathBuf> {
        self.config.custom_binary_path.as_ref()
    }

    pub fn set_custom_binary_path(&mut self, path: Option<PathBuf>) {
        self.config.custom_binary_path = path.clone();
        self.xmrig_adapter.set_custom_binary_path(path);
    }

    pub fn save_profile(&mut self, profile: Profile) {
        if let Some(existing) = self.config.profiles.iter_mut().find(|p| p.id == profile.id) {
            *existing = profile;
        } else {
            self.config.profiles.push(profile);
        }
    }

    pub fn delete_profile(&mut self, profile_id: &str) {
        self.config.profiles.retain(|p| p.id != profile_id);
    }

    pub fn save_config(&self) -> Result<()> {
        self.config.save()
    }

    fn coins_dir(&self) -> PathBuf {
        Self::resolve_coins_dir()
    }

    pub fn list_coins(&self) -> Result<Vec<CoinDefinition>> {
        let coins_dir = self.coins_dir();
        let mut coins = Vec::new();

        if coins_dir.exists() {
            for entry in std::fs::read_dir(&coins_dir)? {
                let entry = entry?;
                let path = entry.path();
                // Skip schema.json and non-JSON files
                if path.extension().map_or(false, |e| e == "json") {
                    let filename = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                    if filename == "schema.json" {
                        continue;
                    }
                    match load_plugin(&path) {
                        Ok((coin, validation)) => {
                            if !validation.trusted {
                                tracing::warn!("Loading untrusted plugin: {}", coin.id);
                            }
                            coins.push(coin);
                        }
                        Err(e) => {
                            tracing::error!("Failed to load plugin {:?}: {}", path, e);
                        }
                    }
                }
            }
        }

        // Sort coins by name for consistent display
        coins.sort_by(|a, b| a.name.cmp(&b.name));

        Ok(coins)
    }

    /// Add a new pool to a coin's JSON file with audit trail
    pub fn add_pool_to_coin(coin_id: &str, pool: PoolConfig) -> Result<CoinDefinition> {
        // Validate pool URL
        let allowed_schemes = ["stratum+tcp://", "stratum+ssl://", "stratum+tls://"];
        let has_valid_scheme = allowed_schemes.iter().any(|s| pool.stratum_url.starts_with(s));
        if !has_valid_scheme {
            return Err(CoreError::PluginValidation(
                "Invalid pool URL. Must start with stratum+tcp://, stratum+ssl://, or stratum+tls://".to_string()
            ));
        }
        
        // Validate URL has host:port format
        let after_scheme = pool.stratum_url.split("://").nth(1).unwrap_or("");
        if !after_scheme.contains(':') {
            return Err(CoreError::PluginValidation(
                "Invalid pool URL. Must include host:port (e.g. pool.example.com:3333)".to_string()
            ));
        }

        // Validate name is not empty
        if pool.name.trim().is_empty() {
            return Err(CoreError::PluginValidation("Pool name cannot be empty".to_string()));
        }

        let coins_dir = Self::resolve_coins_dir();
        let coin_path = coins_dir.join(format!("{}.json", coin_id));
        
        if !coin_path.exists() {
            return Err(CoreError::Miner(format!("Coin '{}' not found", coin_id)));
        }
        
        let content = std::fs::read_to_string(&coin_path)?;
        let mut coin: CoinDefinition = serde_json::from_str(&content)?;
        let previous_count = coin.default_pools.len();
        
        // Check if pool already exists (by URL)
        if coin.default_pools.iter().any(|p| p.stratum_url == pool.stratum_url) {
            return Err(CoreError::PluginValidation(
                format!("Pool with URL '{}' already exists for coin '{}'", pool.stratum_url, coin_id)
            ));
        }
        
        // Check if pool already exists (by name - case insensitive)
        if coin.default_pools.iter().any(|p| p.name.to_lowercase() == pool.name.trim().to_lowercase()) {
            return Err(CoreError::PluginValidation(
                format!("Pool with name '{}' already exists for coin '{}'", pool.name, coin_id)
            ));
        }
        
        let new_pool = PoolConfig {
            name: pool.name.trim().to_string(),
            stratum_url: pool.stratum_url.trim().to_string(),
            tls: pool.tls,
            region: if pool.region.trim().is_empty() { "global".to_string() } else { pool.region.trim().to_string() },
        };
        
        coin.default_pools.push(new_pool);
        let new_count = coin.default_pools.len();
        
        let updated_content = serde_json::to_string_pretty(&coin)?;
        std::fs::write(&coin_path, updated_content)?;
        
        // Write audit trail
        let mut audit = PoolAuditLog::load();
        audit.add_entry(
            coin_id,
            PoolAction::Add,
            &pool.name,
            &pool.stratum_url,
            previous_count,
            new_count,
        );
        
        Ok(coin)
    }

    /// Remove a pool from a coin's JSON file by stratum_url with audit trail
    pub fn remove_pool_from_coin(coin_id: &str, stratum_url: &str) -> Result<CoinDefinition> {
        let coins_dir = Self::resolve_coins_dir();
        let coin_path = coins_dir.join(format!("{}.json", coin_id));
        
        if !coin_path.exists() {
            return Err(CoreError::Miner(format!("Coin '{}' not found", coin_id)));
        }
        
        let content = std::fs::read_to_string(&coin_path)?;
        let mut coin: CoinDefinition = serde_json::from_str(&content)?;
        let previous_count = coin.default_pools.len();
        
        // Find the pool name before removing it (for audit trail)
        let pool_name = coin.default_pools
            .iter()
            .find(|p| p.stratum_url == stratum_url)
            .map(|p| p.name.clone())
            .unwrap_or_else(|| "unknown".to_string());
        
        coin.default_pools.retain(|p| p.stratum_url != stratum_url);
        let new_count = coin.default_pools.len();
        
        if new_count == previous_count {
            return Err(CoreError::PluginValidation(
                format!("Pool with URL '{}' not found for coin '{}'", stratum_url, coin_id)
            ));
        }
        
        let updated_content = serde_json::to_string_pretty(&coin)?;
        std::fs::write(&coin_path, updated_content)?;
        
        // Write audit trail
        let mut audit = PoolAuditLog::load();
        audit.add_entry(
            coin_id,
            PoolAction::Remove,
            &pool_name,
            stratum_url,
            previous_count,
            new_count,
        );
        
        Ok(coin)
    }
    
    /// Resolve coins directory (static version for non-instance methods)
    fn resolve_coins_dir() -> PathBuf {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let project_root = manifest_dir
            .parent()
            .and_then(|p| p.parent());
        
        if let Some(root) = project_root {
            let coins_path = root.join("assets").join("coins");
            if coins_path.exists() {
                return coins_path;
            }
        }
        
        let fallbacks = [
            PathBuf::from("assets/coins"),
            PathBuf::from("../../../assets/coins"),
            dirs::data_dir()
                .map(|d| d.join("openminedash").join("coins"))
                .unwrap_or_default(),
        ];
        
        for path in &fallbacks {
            if path.exists() {
                return path.clone();
            }
        }
        
        PathBuf::from("assets/coins")
    }


    pub async fn start_mining(
        &mut self,
        config: MiningConfig,
        app_handle: tauri::AppHandle,
    ) -> Result<()> {
        if !self.config.consent {
            return Err(CoreError::NoConsent);
        }

        let current_state = self.xmrig_adapter.state();
        if current_state != MinerState::Stopped && current_state != MinerState::Error {
            return Err(CoreError::InvalidState);
        }

        self.status.state = "starting".to_string();

        // Route to appropriate miner based on algorithm
        let routing = route_algorithm(&config.algorithm, config.try_anyway);
        
        let (miner_name, warning) = match routing.miner_type {
            MinerType::XMRig => ("xmrig".to_string(), routing.warning),
            MinerType::CpuminerOpt => ("cpuminer-opt".to_string(), routing.warning),
            MinerType::External => {
                if config.try_anyway {
                    // "Try Anyway" mode: route External to cpuminer-opt as best-effort
                    self.active_miner = ActiveMiner::CpuminerOpt;
                    let adapter_config: AdapterMiningConfig = config.clone().into();
                    let child = self.cpuminer_adapter
                        .start(&adapter_config, app_handle)
                        .await
                        .map_err(|e| CoreError::Miner(e.to_string()))?;
                    self.miner_process = Some(child);
                    let now = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs();
                    self.status = MiningStatus {
                        state: "running".to_string(),
                        is_running: true,
                        coin: Some(config.coin),
                        pool: Some(config.pool.clone()),
                        wallet: Some(config.wallet),
                        worker: Some(config.worker),
                        algorithm: Some(config.algorithm),
                        active_miner: "cpuminer-opt".to_string(),
                        warning: Some("GPU/ASIC algorithm forced on CPU. This will be extremely slow and likely not profitable.".to_string()),
                        started_at: now,
                        ..Default::default()
                    };
                    return Ok(());
                }
                self.status.state = "stopped".to_string();
                return Err(CoreError::Miner(
                    routing.warning.unwrap_or_else(|| "This algorithm requires GPU/ASIC hardware. Use 'Try Mining Anyway' to attempt CPU mining.".to_string())
                ));
            }
            MinerType::Unsupported => {
                self.status.state = "stopped".to_string();
                return Err(CoreError::Miner(
                    routing.warning.unwrap_or_else(|| "No CPU miner available for this algorithm.".to_string())
                ));
            }
        };

        // Create lock file for crash recovery
        let session = MiningSession {
            coin: config.coin.clone(),
            pool: config.pool.clone(),
            wallet: config.wallet.clone(),
            worker: config.worker.clone(),
            started_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            pid: std::process::id(),
        };
        let _ = create_mining_lock(&session);

        let adapter_config: AdapterMiningConfig = config.clone().into();
        
        // Start the appropriate miner
        let child = match routing.miner_type {
            MinerType::XMRig => {
                self.active_miner = ActiveMiner::XMRig;
                self.xmrig_adapter
                    .start(&adapter_config, app_handle)
                    .await
                    .map_err(|e| CoreError::Miner(e.to_string()))?
            }
            MinerType::CpuminerOpt => {
                self.active_miner = ActiveMiner::CpuminerOpt;
                self.cpuminer_adapter
                    .start(&adapter_config, app_handle)
                    .await
                    .map_err(|e| CoreError::Miner(e.to_string()))?
            }
            _ => unreachable!(),
        };

        self.miner_process = Some(child);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        self.status = MiningStatus {
            state: "running".to_string(),
            is_running: true,
            coin: Some(config.coin),
            pool: Some(config.pool.clone()),
            wallet: Some(config.wallet),
            worker: Some(config.worker),
            algorithm: Some(config.algorithm),
            active_miner: miner_name,
            warning,
            started_at: now,
            ..Default::default()
        };

        Ok(())
    }

    pub async fn stop_mining(&mut self) -> Result<()> {
        // Check if any miner is running
        let xmrig_running = self.xmrig_adapter.state() == MinerState::Running;
        let cpuminer_running = self.cpuminer_adapter.state() == MinerState::Running;
        
        if !xmrig_running && !cpuminer_running {
            return Ok(());
        }

        // Save mining record before stopping
        if self.status.is_running && self.status.started_at > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            
            let record = MiningRecord {
                id: uuid::Uuid::new_v4().to_string(),
                coin: self.status.coin.clone().unwrap_or_default(),
                symbol: self.status.coin.clone().unwrap_or_default().to_uppercase(),
                pool: self.status.pool.clone().unwrap_or_default(),
                wallet: self.status.wallet.clone().unwrap_or_default(),
                worker: self.status.worker.clone().unwrap_or_default(),
                started_at: self.status.started_at,
                ended_at: now,
                duration_secs: now.saturating_sub(self.status.started_at),
                accepted_shares: self.status.accepted_shares,
                rejected_shares: self.status.rejected_shares,
                avg_hashrate: self.status.avg_hashrate,
                algorithm: self.status.algorithm.clone().unwrap_or_default(),
            };
            
            if record.duration_secs > 10 { // Only save sessions longer than 10 seconds
                self.mining_history.add_record(record);
            }
        }

        self.status.state = "stopping".to_string();

        if let Some(mut child) = self.miner_process.take() {
            match self.active_miner {
                ActiveMiner::XMRig => self.xmrig_adapter.stop(&mut child).await,
                ActiveMiner::CpuminerOpt => self.cpuminer_adapter.stop(&mut child).await,
                ActiveMiner::None => {}
            }
        }

        // Remove lock file for clean shutdown
        remove_mining_lock();

        self.active_miner = ActiveMiner::None;
        self.status = MiningStatus {
            state: "stopped".to_string(),
            ..Default::default()
        };
        Ok(())
    }

    pub async fn refresh_stats(&mut self) -> Result<()> {
        // Calculate elapsed time since mining started (independent of pool connection)
        if self.status.is_running && self.status.started_at > 0 {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            let elapsed = now.saturating_sub(self.status.started_at);
            // Use elapsed time if XMRig uptime is 0 (not connected to pool yet)
            if self.status.uptime == 0 {
                self.status.uptime = elapsed;
            }
        }

        match self.active_miner {
            ActiveMiner::XMRig => {
                if self.xmrig_adapter.state() != MinerState::Running {
                    tracing::debug!("XMRig not running, skipping stats refresh");
                    return Ok(());
                }
                match self.xmrig_adapter.get_stats().await {
                    Ok(stats) => {
                        self.status.hashrate = stats.current_hashrate();
                        self.status.avg_hashrate = stats.avg_hashrate();
                        self.status.accepted_shares = stats.accepted_shares();
                        self.status.rejected_shares = stats.rejected_shares();
                        // Use XMRig's uptime if connected, otherwise keep elapsed time
                        if stats.connection.uptime > 0 {
                            self.status.uptime = stats.connection.uptime;
                        }
                        tracing::debug!(
                            "XMRig stats: hashrate={}, accepted={}, uptime={}",
                            self.status.hashrate,
                            self.status.accepted_shares,
                            self.status.uptime
                        );
                    }
                    Err(e) => {
                        tracing::warn!("Failed to get XMRig stats: {}", e);
                    }
                }
            }
            ActiveMiner::CpuminerOpt => {
                if self.cpuminer_adapter.state() != MinerState::Running {
                    return Ok(());
                }
                let stats = self.cpuminer_adapter.get_stats();
                self.status.hashrate = stats.hashrate;
                self.status.avg_hashrate = stats.avg_hashrate;
                self.status.accepted_shares = stats.accepted;
                self.status.rejected_shares = stats.rejected;
                self.status.uptime = stats.uptime;
            }
            ActiveMiner::None => {}
        }

        Ok(())
    }

    // Remote monitoring
    pub fn add_remote_endpoint(&mut self, endpoint: RemoteEndpoint) {
        self.remote_endpoints.push(endpoint);
    }

    pub fn remote_endpoints(&self) -> &[RemoteEndpoint] {
        &self.remote_endpoints
    }

    pub async fn fetch_remote_stats(&self, endpoint_id: &str) -> Option<RemoteMinerStats> {
        self.remote_endpoints
            .iter()
            .find(|e| e.id == endpoint_id)
            .map(|e| fetch_remote_stats(e))
            .map(|f| futures::executor::block_on(f))
    }

    // Mining history
    pub fn mining_history(&self) -> &MiningHistory {
        &self.mining_history
    }

    pub fn get_history_summary(&self) -> HistorySummary {
        self.mining_history.get_summary()
    }

    pub fn get_history_records(&self) -> &[MiningRecord] {
        &self.mining_history.records
    }

    pub fn clear_mining_history(&mut self) {
        self.mining_history.clear();
    }

    /// Change performance preset while mining is running
    pub async fn change_preset(&mut self, preset: PerformancePreset) -> Result<()> {
        if !self.status.is_running {
            return Err(CoreError::NotRunning);
        }

        match self.active_miner {
            ActiveMiner::XMRig => {
                self.xmrig_adapter
                    .set_preset(preset)
                    .await
                    .map_err(|e| CoreError::Miner(e.to_string()))?;
                tracing::info!("Changed XMRig preset to {:?}", preset);
                Ok(())
            }
            ActiveMiner::CpuminerOpt => {
                // cpuminer-opt doesn't support runtime thread changes
                // Would need to restart
                Err(CoreError::Miner(
                    "cpuminer-opt doesn't support runtime preset changes. Stop and restart mining.".to_string()
                ))
            }
            ActiveMiner::None => Err(CoreError::NotRunning),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

// PerformancePreset is already imported at the top of this file
