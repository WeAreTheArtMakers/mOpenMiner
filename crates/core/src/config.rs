use crate::{Profile, Result, ThreadBudgetSettings};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Application behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorSettings {
    /// Stop all mining when app quits [DEFAULT: true]
    pub quit_stops_mining: bool,
    /// Close window hides app instead of quitting [DEFAULT: true]
    pub close_hides_app: bool,
}

impl Default for BehaviorSettings {
    fn default() -> Self {
        Self {
            quit_stops_mining: true,
            close_hides_app: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub consent: bool,
    pub theme: String,
    pub profiles: Vec<Profile>,
    pub custom_binary_path: Option<PathBuf>,
    #[serde(default)]
    pub thread_budget: ThreadBudgetSettings,
    #[serde(default)]
    pub behavior: BehaviorSettings,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            consent: false,
            theme: "dark".to_string(),
            profiles: Vec::new(),
            custom_binary_path: None,
            thread_budget: ThreadBudgetSettings::default(),
            behavior: BehaviorSettings::default(),
        }
    }
}

impl AppConfig {
    pub fn config_dir() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("openminedash")
    }

    pub fn config_path() -> PathBuf {
        Self::config_dir().join("config.json")
    }

    pub fn load() -> Result<Self> {
        let path = Self::config_path();
        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let config = serde_json::from_str(&content)?;
            Ok(config)
        } else {
            Ok(Self::default())
        }
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }
}
