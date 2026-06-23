//! Pool API integration for fetching miner balances

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolBalance {
    pub pool_name: String,
    pub pending_balance: f64,
    pub total_paid: f64,
    pub min_payout: f64,
    pub symbol: String,
    pub last_payment: Option<u64>,
    pub hashrate: Option<f64>,
}

// ============================================================================
// Pool Discovery - Well-known pool API integration
// ============================================================================

/// Discovered pool from external API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPool {
    pub name: String,
    pub url: String,
    pub tls: bool,
}

/// Result from miningpoolstats.co.uk discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResult {
    pub coin_id: String,
    pub coin_name: String,
    pub pools: Vec<DiscoveredPool>,
}

/// Discover pools for a given coin symbol using miningpoolstats.co.uk
pub async fn discover_pools(symbol: &str) -> Vec<DiscoveredPool> {
    let symbol_upper = symbol.to_uppercase();
    
    // Try multiple API endpoints
    let urls = [
        format!("https://api.miningpoolstats.co.uk/{}", symbol_upper),
        format!("https://api.miningpoolstats.stream/{}", symbol_upper),
        format!("https://miningpoolstats.stream/{}", symbol_upper),
    ];
    
    for url in &urls {
        let result = try_discover(url).await;
        if !result.is_empty() {
            return result;
        }
    }
    
    Vec::new()
}

/// Try a single API endpoint for pool discovery
async fn try_discover(url: &str) -> Vec<DiscoveredPool> {
    // Extract symbol from URL for logging
    let symbol = url.rsplit('/').next().unwrap_or("unknown");
    
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .user_agent("OpenMiner/1.0")
        .build()
        .unwrap_or_default();
    
    let resp = match client.get(url).send().await {
        Ok(r) => r,
        Err(e) => {
            tracing::warn!("Failed to fetch pool data for {}: {}", symbol, e);
            return Vec::new();
        }
    };
    
    if !resp.status().is_success() {
        tracing::warn!("API returned status {} for {}", resp.status(), symbol);
        return Vec::new();
    }
    
    let data: serde_json::Value = match resp.json().await {
        Ok(d) => d,
        Err(e) => {
            tracing::warn!("Failed to parse pool data for {}: {}", symbol, e);
            return Vec::new();
        }
    };
    
    // miningpoolstats.co.uk returns an object with pools array
    // Structure: { "pools": [ { "url": "...", "name": "...", "tls": bool }, ... ] }
    let pools = match data.get("pools").and_then(|p| p.as_array()) {
        Some(arr) => arr,
        None => {
            // Try alternative format: direct array of pool objects
            if let Some(arr) = data.as_array() {
                arr
            } else {
                tracing::debug!("No pools array in response for {}", symbol);
                return Vec::new();
            }
        }
    };
    
    pools.iter().filter_map(|pool| {
        let url = pool.get("url")
            .or_else(|| pool.get("stratum_url"))
            .and_then(|u| u.as_str())?;
        
        // Convert pool URL to stratum format if needed
        let stratum_url = if url.starts_with("stratum+") {
            url.to_string()
        } else if url.contains(':') {
            // Assume host:port format, wrap in stratum+tcp://
            format!("stratum+tcp://{}", url)
        } else {
            return None;
        };
        
        let name = pool.get("name")
            .or_else(|| pool.get("pool"))
            .and_then(|n| n.as_str())
            .unwrap_or("Unknown Pool")
            .to_string();
        
        let tls = pool.get("tls")
            .or_else(|| pool.get("ssl"))
            .and_then(|t| t.as_bool())
            .unwrap_or(false);
        
        Some(DiscoveredPool {
            name,
            url: stratum_url,
            tls,
        })
    }).collect()
}

/// Discover pools with fallback to hardcoded known pools for popular coins
pub async fn discover_pools_with_fallback(symbol: &str) -> Vec<DiscoveredPool> {
    let pools = discover_pools(symbol).await;
    
    if !pools.is_empty() {
        return pools;
    }
    
    // Fallback: return hardcoded known pools for popular coins
    match symbol.to_uppercase().as_str() {
        "XMR" => vec![
            DiscoveredPool { name: "MoneroOcean".into(), url: "stratum+tcp://gulf.moneroocean.stream:10128".into(), tls: false },
            DiscoveredPool { name: "SupportXMR".into(), url: "stratum+tcp://pool.supportxmr.com:3333".into(), tls: false },
            DiscoveredPool { name: "Nanopool".into(), url: "stratum+tcp://xmr-usa.dwarfpool.com:4444".into(), tls: false },
        ],
        "BTC" => vec![
            DiscoveredPool { name: "F2Pool".into(), url: "stratum+tcp://btc.f2pool.com:1318".into(), tls: false },
            DiscoveredPool { name: "ViaBTC".into(), url: "stratum+tcp://btc.viabtc.com:3333".into(), tls: false },
        ],
        "LTC" => vec![
            DiscoveredPool { name: "F2Pool".into(), url: "stratum+tcp://ltc.f2pool.com:1318".into(), tls: false },
            DiscoveredPool { name: "ViaBTC".into(), url: "stratum+tcp://ltc.viabtc.com:443".into(), tls: true },
        ],
        "DOGE" => vec![
            DiscoveredPool { name: "F2Pool".into(), url: "stratum+tcp://doge.f2pool.com:1318".into(), tls: false },
            DiscoveredPool { name: "ViaBTC".into(), url: "stratum+tcp://doge.viabtc.com:3333".into(), tls: false },
        ],
        "ZEC" => vec![
            DiscoveredPool { name: "F2Pool".into(), url: "stratum+tcp://zec.f2pool.com:1423".into(), tls: false },
            DiscoveredPool { name: "ViaBTC".into(), url: "stratum+tcp://zec.viabtc.com:3333".into(), tls: false },
        ],
        "ETC" => vec![
            DiscoveredPool { name: "Nanopool".into(), url: "stratum+ssl://etc.nanopool.org:9443".into(), tls: true },
            DiscoveredPool { name: "2Miners".into(), url: "stratum+ssl://etc.2miners.com:1100".into(), tls: true },
        ],
        "RVN" => vec![
            DiscoveredPool { name: "F2Pool".into(), url: "stratum+tcp://raven.f2pool.com:3636".into(), tls: false },
            DiscoveredPool { name: "2Miners".into(), url: "stratum+ssl://rvn.2miners.com:4444".into(), tls: true },
        ],
        "ERG" => vec![
            DiscoveredPool { name: "HeroMiners".into(), url: "stratum+ssl://ergo.herominers.com:1180".into(), tls: true },
            DiscoveredPool { name: "2Miners".into(), url: "stratum+ssl://erg.2miners.com:4444".into(), tls: true },
        ],
        "FLUX" => vec![
            DiscoveredPool { name: "2Miners".into(), url: "stratum+ssl://flux.2miners.com:4444".into(), tls: true },
        ],
        "KAS" => vec![
            DiscoveredPool { name: "Woolypooly".into(), url: "stratum+tcp://pool.woolypooly.com:3112".into(), tls: false },
            DiscoveredPool { name: "HeroMiners".into(), url: "stratum+ssl://kaspa.herominers.com:1206".into(), tls: true },
        ],
        "RTM" => vec![
            DiscoveredPool { name: "R-Pool".into(), url: "stratum+tcp://r-pool.net:3008".into(), tls: false },
            DiscoveredPool { name: "Vipor".into(), url: "stratum+tcp://pool.vipor.net:5066".into(), tls: false },
            DiscoveredPool { name: "Mining Dutch".into(), url: "stratum+tcp://rtm.miningdutch.com:3333".into(), tls: false },
            DiscoveredPool { name: "2Miners".into(), url: "stratum+ssl://rtm.2miners.com:4444".into(), tls: true },
            DiscoveredPool { name: "Kryptex".into(), url: "stratum+tcp://rtm.kryptex.network:7777".into(), tls: false },
            DiscoveredPool { name: "HeroMiners".into(), url: "stratum+ssl://raptoreum.herominers.com:1196".into(), tls: true },
            DiscoveredPool { name: "MinerPool".into(), url: "stratum+tcp://rtm.minerpool.org:6248".into(), tls: false },
        ],
        "XMG" | "MAG" | "GRS" | "GRFT" | "XVG" | "DASH" => {
            // Generic ghostrider pools for other ghostrider coins
            vec![
                DiscoveredPool { name: "R-Pool".into(), url: "stratum+tcp://r-pool.net:3008".into(), tls: false },
                DiscoveredPool { name: "Vipor".into(), url: "stratum+tcp://pool.vipor.net:5066".into(), tls: false },
            ]
        }
        _ => Vec::new(),
    }
}

/// Fetch balance from MoneroOcean
pub async fn fetch_moneroocean_balance(wallet: &str) -> Result<PoolBalance, String> {
    let url = format!("https://api.moneroocean.stream/miner/{}/stats", wallet);
    
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !resp.status().is_success() {
        return Err(format!("API returned status: {}", resp.status()));
    }
    
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // MoneroOcean returns balance in atomic units (piconero = 1e-12 XMR)
    let pending = data["amtDue"].as_f64().unwrap_or(0.0) / 1e12;
    let paid = data["amtPaid"].as_f64().unwrap_or(0.0) / 1e12;
    let hashrate = data["hash"].as_f64();
    
    Ok(PoolBalance {
        pool_name: "MoneroOcean".to_string(),
        pending_balance: pending,
        total_paid: paid,
        min_payout: 0.003,
        symbol: "XMR".to_string(),
        last_payment: None,
        hashrate,
    })
}

/// Fetch balance from SupportXMR
pub async fn fetch_supportxmr_balance(wallet: &str) -> Result<PoolBalance, String> {
    let url = format!("https://supportxmr.com/api/miner/{}/stats", wallet);
    
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !resp.status().is_success() {
        return Err(format!("API returned status: {}", resp.status()));
    }
    
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    // SupportXMR returns balance in atomic units
    let pending = data["amtDue"].as_f64().unwrap_or(0.0) / 1e12;
    let paid = data["amtPaid"].as_f64().unwrap_or(0.0) / 1e12;
    let hashrate = data["hash"].as_f64();
    
    Ok(PoolBalance {
        pool_name: "SupportXMR".to_string(),
        pending_balance: pending,
        total_paid: paid,
        min_payout: 0.1,
        symbol: "XMR".to_string(),
        last_payment: None,
        hashrate,
    })
}

/// Fetch balance from Nanopool XMR
pub async fn fetch_nanopool_balance(wallet: &str) -> Result<PoolBalance, String> {
    let url = format!("https://api.nanopool.org/v1/xmr/balance/{}", wallet);
    
    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(10))
        .send()
        .await
        .map_err(|e| format!("Request failed: {}", e))?;
    
    if !resp.status().is_success() {
        return Err(format!("API returned status: {}", resp.status()));
    }
    
    let data: serde_json::Value = resp
        .json()
        .await
        .map_err(|e| format!("Failed to parse JSON: {}", e))?;
    
    let pending = data["data"].as_f64().unwrap_or(0.0);
    
    Ok(PoolBalance {
        pool_name: "Nanopool".to_string(),
        pending_balance: pending,
        total_paid: 0.0, // Would need separate API call
        min_payout: 0.1,
        symbol: "XMR".to_string(),
        last_payment: None,
        hashrate: None,
    })
}

/// Generic pool balance fetcher
pub async fn fetch_pool_balance(pool_host: &str, wallet: &str) -> Result<PoolBalance, String> {
    match pool_host {
        "gulf.moneroocean.stream" => fetch_moneroocean_balance(wallet).await,
        "pool.supportxmr.com" => fetch_supportxmr_balance(wallet).await,
        "xmr.nanopool.org" => fetch_nanopool_balance(wallet).await,
        _ => Err(format!("Pool API not supported: {}", pool_host)),
    }
}
