//! In-app benchmark for measuring hashrate on user's hardware.

use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::info;

const BENCHMARK_DURATION_SECS: u64 = 60;
const SAMPLE_INTERVAL_SECS: u64 = 5;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    pub duration_secs: u64,
    pub samples: Vec<f64>,
    pub avg_hashrate: f64,
    pub peak_hashrate: f64,
    pub min_hashrate: f64,
    pub recommended_preset: String,
    pub recommended_threads: u32,
    pub hardware_info: HardwareInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    pub cpu_brand: String,
    pub cpu_cores: u32,
    pub cpu_threads: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkProgress {
    pub elapsed_secs: u64,
    pub total_secs: u64,
    pub current_hashrate: f64,
    pub samples_collected: usize,
}

impl BenchmarkResult {
    /// Generate recommendation based on benchmark results
    pub fn generate_recommendation(avg_hashrate: f64, hardware: &HardwareInfo) -> (String, u32) {
        // Simple heuristic based on hashrate per thread
        let hashrate_per_thread = avg_hashrate / hardware.cpu_threads as f64;
        
        // M1 Pro typically gets ~150-200 H/s per thread at balanced
        let (preset, thread_ratio) = if hashrate_per_thread > 180.0 {
            ("max", 0.75)
        } else if hashrate_per_thread > 120.0 {
            ("balanced", 0.5)
        } else {
            ("eco", 0.25)
        };
        
        let recommended_threads = ((hardware.cpu_threads as f64) * thread_ratio).max(1.0) as u32;
        
        (preset.to_string(), recommended_threads)
    }
}

/// Get hardware info for benchmark context
pub fn get_hardware_info() -> HardwareInfo {
    HardwareInfo {
        cpu_brand: get_cpu_brand(),
        cpu_cores: num_cpus::get_physical() as u32,
        cpu_threads: num_cpus::get() as u32,
    }
}

fn get_cpu_brand() -> String {
    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("sysctl")
            .args(["-n", "machdep.cpu.brand_string"])
            .output()
            .ok()
            .and_then(|o| String::from_utf8(o.stdout).ok())
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Unknown".to_string()
    }
}

/// Expected hashrates for Apple Silicon (for UI display)
pub fn get_expected_hashrates() -> Vec<(&'static str, u32, u32, u32)> {
    // (chip, eco, balanced, max)
    vec![
        ("M1", 400, 800, 1200),
        ("M1 Pro", 600, 1200, 1800),
        ("M1 Max", 800, 1600, 2400),
        ("M1 Ultra", 1500, 3000, 4500),
        ("M2", 450, 900, 1350),
        ("M2 Pro", 700, 1400, 2100),
        ("M2 Max", 900, 1800, 2700),
        ("M3", 500, 1000, 1500),
        ("M3 Pro", 750, 1500, 2250),
        ("M3 Max", 1000, 2000, 3000),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommendation_generation() {
        let hardware = HardwareInfo {
            cpu_brand: "Apple M1 Pro".to_string(),
            cpu_cores: 8,
            cpu_threads: 10,
        };

        // High performance
        let (preset, threads) = BenchmarkResult::generate_recommendation(2000.0, &hardware);
        assert_eq!(preset, "max");
        assert!(threads >= 7);

        // Medium performance
        let (preset, threads) = BenchmarkResult::generate_recommendation(1400.0, &hardware);
        assert_eq!(preset, "balanced");

        // Low performance
        let (preset, threads) = BenchmarkResult::generate_recommendation(800.0, &hardware);
        assert_eq!(preset, "eco");
    }

    #[test]
    fn test_hardware_info() {
        let info = get_hardware_info();
        assert!(info.cpu_cores > 0);
        assert!(info.cpu_threads >= info.cpu_cores);
    }
}
