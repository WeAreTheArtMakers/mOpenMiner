use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

const MAX_LOG_LINES: usize = 2000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticsExport {
    pub warning: String,
    pub app_version: String,
    pub os_version: String,
    pub architecture: String,
    pub config_masked: MaskedConfig,
    pub logs: Vec<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedConfig {
    pub consent: bool,
    pub theme: String,
    pub profiles: Vec<MaskedProfile>,
    pub custom_binary_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaskedProfile {
    pub name: String,
    pub coin: String,
    pub pool: String,
    pub wallet_masked: String,
    pub worker: String,
    pub threads: u32,
}

pub struct LogBuffer {
    lines: VecDeque<String>,
}

impl LogBuffer {
    pub fn new() -> Self {
        Self {
            lines: VecDeque::with_capacity(MAX_LOG_LINES),
        }
    }

    pub fn push(&mut self, line: String) {
        if self.lines.len() >= MAX_LOG_LINES {
            self.lines.pop_front();
        }
        self.lines.push_back(line);
    }

    pub fn get_all(&self) -> Vec<String> {
        self.lines.iter().cloned().collect()
    }

    pub fn clear(&mut self) {
        self.lines.clear();
    }
}

impl Default for LogBuffer {
    fn default() -> Self {
        Self::new()
    }
}

/// Mask wallet address for privacy (show first 6 and last 4 chars)
pub fn mask_wallet(wallet: &str) -> String {
    if wallet.len() <= 12 {
        return "***".to_string();
    }
    format!("{}...{}", &wallet[..6], &wallet[wallet.len() - 4..])
}

/// IMPORTANT: Diagnostics export may contain sensitive metadata.
/// Wallet addresses are masked by default for privacy.
pub fn create_diagnostics_export(
    config: &crate::AppConfig,
    logs: Vec<String>,
    mask_wallets: bool,
) -> DiagnosticsExport {
    let profiles: Vec<MaskedProfile> = config
        .profiles
        .iter()
        .map(|p| MaskedProfile {
            name: p.name.clone(),
            coin: p.coin.clone(),
            pool: p.pool.clone(),
            wallet_masked: if mask_wallets {
                mask_wallet(&p.wallet)
            } else {
                p.wallet.clone()
            },
            worker: p.worker.clone(),
            threads: p.threads,
        })
        .collect();

    DiagnosticsExport {
        warning: "This file may contain sensitive metadata. Review before sharing.".to_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
        os_version: get_os_version(),
        architecture: std::env::consts::ARCH.to_string(),
        config_masked: MaskedConfig {
            consent: config.consent,
            theme: config.theme.clone(),
            profiles,
            custom_binary_path: config.custom_binary_path.as_ref().map(|p| p.display().to_string()),
        },
        logs,
        timestamp: chrono_lite_timestamp(),
    }
}

fn get_os_version() -> String {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sw_vers")
            .arg("-productVersion")
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| format!("macOS {}", s.trim()))
            .unwrap_or_else(|| "macOS (unknown version)".to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        std::env::consts::OS.to_string()
    }
}

fn chrono_lite_timestamp() -> String {
    // Simple timestamp without chrono dependency
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mask_wallet() {
        assert_eq!(mask_wallet("48edfHu7V9Z84YzzMa6fUueoELZ9ZRXq9VetWzYGzKt52XU5xvqgzYnDK9URnRoJMk1j8nLwEVsaSWJ4fhdUyZijBGUicoD"), "48edfH...icoD");
        assert_eq!(mask_wallet("short"), "***");
    }

    #[test]
    fn test_log_buffer() {
        let mut buffer = LogBuffer::new();
        for i in 0..2500 {
            buffer.push(format!("line {}", i));
        }
        assert_eq!(buffer.get_all().len(), MAX_LOG_LINES);
        assert!(buffer.get_all().last().unwrap().contains("2499"));
    }
}
