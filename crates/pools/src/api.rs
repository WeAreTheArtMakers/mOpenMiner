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
