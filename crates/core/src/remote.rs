//! Remote miner monitoring for external ASIC/rigs.
//! Read-only: no control commands sent to remote devices.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use tracing::{info, warn};

/// Remote miner endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteEndpoint {
    pub id: String,
    pub name: String,
    pub url: String,
    pub api_type: RemoteApiType,
    pub poll_interval_secs: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RemoteApiType {
    /// CGMiner/BFGMiner compatible API
    CgMiner,
    /// Antminer/Bitmain web API
    Antminer,
    /// Generic JSON stats endpoint
    JsonStats,
}

/// Stats from remote miner (normalized)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RemoteMinerStats {
    pub online: bool,
    pub hashrate: f64,
    pub hashrate_unit: String,
    pub accepted_shares: u64,
    pub rejected_shares: u64,
    pub uptime_secs: u64,
    pub pool: Option<String>,
    pub worker: Option<String>,
    pub temperature: Option<f64>,
    pub fan_speed: Option<u32>,
    pub error: Option<String>,
    pub last_updated: u64,
}

/// Fetch stats from remote miner (read-only)
pub async fn fetch_remote_stats(endpoint: &RemoteEndpoint) -> RemoteMinerStats {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()
        .unwrap_or_default();

    match endpoint.api_type {
        RemoteApiType::CgMiner => fetch_cgminer_stats(&client, &endpoint.url).await,
        RemoteApiType::Antminer => fetch_antminer_stats(&client, &endpoint.url).await,
        RemoteApiType::JsonStats => fetch_json_stats(&client, &endpoint.url).await,
    }
}

async fn fetch_cgminer_stats(client: &reqwest::Client, url: &str) -> RemoteMinerStats {
    // CGMiner uses a simple JSON-RPC over TCP, but for HTTP wrapper:
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    match client.get(format!("{}/summary", url)).send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                RemoteMinerStats {
                    online: true,
                    hashrate: data["SUMMARY"][0]["GHS 5s"].as_f64().unwrap_or(0.0) * 1_000_000_000.0,
                    hashrate_unit: "H/s".to_string(),
                    accepted_shares: data["SUMMARY"][0]["Accepted"].as_u64().unwrap_or(0),
                    rejected_shares: data["SUMMARY"][0]["Rejected"].as_u64().unwrap_or(0),
                    uptime_secs: data["SUMMARY"][0]["Elapsed"].as_u64().unwrap_or(0),
                    pool: data["SUMMARY"][0]["Pool URL"].as_str().map(|s| s.to_string()),
                    worker: None,
                    temperature: None,
                    fan_speed: None,
                    error: None,
                    last_updated: now,
                }
            } else {
                error_stats("Failed to parse response", now)
            }
        }
        Err(e) => error_stats(&e.to_string(), now),
    }
}

async fn fetch_antminer_stats(client: &reqwest::Client, url: &str) -> RemoteMinerStats {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    // Antminer typically requires auth, this is a simplified version
    match client.get(format!("{}/cgi-bin/stats.cgi", url)).send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                RemoteMinerStats {
                    online: true,
                    hashrate: data["rate_5s"].as_f64().unwrap_or(0.0),
                    hashrate_unit: "TH/s".to_string(),
                    accepted_shares: 0,
                    rejected_shares: 0,
                    uptime_secs: data["elapsed"].as_u64().unwrap_or(0),
                    pool: None,
                    worker: None,
                    temperature: data["temp"].as_f64(),
                    fan_speed: data["fan"].as_u64().map(|f| f as u32),
                    error: None,
                    last_updated: now,
                }
            } else {
                error_stats("Failed to parse response", now)
            }
        }
        Err(e) => error_stats(&e.to_string(), now),
    }
}

async fn fetch_json_stats(client: &reqwest::Client, url: &str) -> RemoteMinerStats {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    match client.get(url).send().await {
        Ok(resp) => {
            if let Ok(data) = resp.json::<serde_json::Value>().await {
                RemoteMinerStats {
                    online: true,
                    hashrate: data["hashrate"].as_f64().unwrap_or(0.0),
                    hashrate_unit: data["hashrate_unit"].as_str().unwrap_or("H/s").to_string(),
                    accepted_shares: data["accepted"].as_u64().unwrap_or(0),
                    rejected_shares: data["rejected"].as_u64().unwrap_or(0),
                    uptime_secs: data["uptime"].as_u64().unwrap_or(0),
                    pool: data["pool"].as_str().map(|s| s.to_string()),
                    worker: data["worker"].as_str().map(|s| s.to_string()),
                    temperature: data["temperature"].as_f64(),
                    fan_speed: data["fan_speed"].as_u64().map(|f| f as u32),
                    error: None,
                    last_updated: now,
                }
            } else {
                error_stats("Failed to parse response", now)
            }
        }
        Err(e) => error_stats(&e.to_string(), now),
    }
}

fn error_stats(error: &str, timestamp: u64) -> RemoteMinerStats {
    RemoteMinerStats {
        online: false,
        error: Some(error.to_string()),
        last_updated: timestamp,
        ..Default::default()
    }
}
