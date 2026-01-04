//! Plugin validation and security for coin definitions.

use crate::{CoinDefinition, CoreError, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::path::Path;

/// Trusted plugin registry - hashes of built-in plugins
const TRUSTED_PLUGIN_HASHES: &[(&str, &str)] = &[
    ("xmr", "HASH_PLACEHOLDER_XMR"),
    ("btc", "HASH_PLACEHOLDER_BTC"),
    ("ltc", "HASH_PLACEHOLDER_LTC"),
];

/// URL schemes that are blocked in pool URLs
const BLOCKED_URL_SCHEMES: &[&str] = &["file://", "javascript:", "data:"];

/// Allowed URL schemes for stratum
const ALLOWED_URL_SCHEMES: &[&str] = &["stratum+tcp://", "stratum+ssl://", "stratum+tls://"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginValidationResult {
    pub valid: bool,
    pub trusted: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Validate a coin plugin definition
pub fn validate_plugin(coin: &CoinDefinition, content_hash: &str) -> PluginValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check schema version
    if coin.schema_version != 1 {
        errors.push(format!("Unsupported schema_version: {}", coin.schema_version));
    }

    // Validate ID format
    if !coin.id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        errors.push("ID must be lowercase alphanumeric with hyphens only".to_string());
    }

    // Validate symbol
    if !coin.symbol.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit()) {
        errors.push("Symbol must be uppercase alphanumeric".to_string());
    }

    // Validate algorithm
    let valid_algos = [
        "randomx",      // Monero
        "sha256",       // Bitcoin
        "scrypt",       // Litecoin, Dogecoin
        "ethash",       // Ethereum (legacy)
        "etchash",      // Ethereum Classic
        "kawpow",       // Ravencoin
        "kheavyhash",   // Kaspa
        "autolykos2",   // Ergo
        "equihash",     // Zcash
        "zelHash",      // Flux
        "ghostrider",   // Raptoreum
        "verushash",    // Verus
        "blake3",       // Various
        "x11",          // Dash
        "x16r",         // Various
        "cuckoo",       // Grin
    ];
    if !valid_algos.contains(&coin.algorithm.as_str()) {
        errors.push(format!("Unknown algorithm: {}", coin.algorithm));
    }

    // Validate miner type
    let valid_miners = [
        "xmrig",         // CPU miner for RandomX
        "external-asic", // External ASIC hardware
        "external-gpu",  // External GPU miner
        "custom",        // Custom miner binary
    ];
    if !valid_miners.contains(&coin.recommended_miner.as_str()) {
        errors.push(format!("Unknown miner type: {}", coin.recommended_miner));
    }

    // Validate pools
    for pool in &coin.default_pools {
        // Check URL scheme
        let has_valid_scheme = ALLOWED_URL_SCHEMES.iter().any(|s| pool.stratum_url.starts_with(s));
        if !has_valid_scheme {
            errors.push(format!("Invalid URL scheme in pool: {}", pool.stratum_url));
        }

        // Check for blocked schemes
        for blocked in BLOCKED_URL_SCHEMES {
            if pool.stratum_url.to_lowercase().contains(blocked) {
                errors.push(format!("Blocked URL scheme in pool: {}", pool.stratum_url));
            }
        }

        // Check for localhost (warning, not error)
        if pool.stratum_url.contains("localhost") || pool.stratum_url.contains("127.0.0.1") {
            warnings.push(format!("Pool uses localhost: {}", pool.stratum_url));
        }

        // Validate TLS flag matches URL
        let url_has_ssl = pool.stratum_url.contains("+ssl") || pool.stratum_url.contains("+tls");
        if pool.tls != url_has_ssl {
            warnings.push(format!("TLS flag mismatch for pool: {}", pool.name));
        }
    }

    // Check if trusted (built-in plugins or marked as trusted)
    let is_builtin = TRUSTED_PLUGIN_HASHES
        .iter()
        .any(|(id, hash)| *id == coin.id && *hash == content_hash);
    
    let trusted = is_builtin || coin.trusted;

    // Untrusted plugins get a warning
    if !trusted {
        warnings.push("This is an untrusted plugin. Verify the source before using.".to_string());
    }

    PluginValidationResult {
        valid: errors.is_empty(),
        trusted,
        errors,
        warnings,
    }
}

/// Compute hash of plugin content for trust verification
pub fn compute_plugin_hash(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    hex::encode(hasher.finalize())
}

/// Load and validate a plugin from file
pub fn load_plugin(path: &Path) -> Result<(CoinDefinition, PluginValidationResult)> {
    let content = std::fs::read_to_string(path)?;
    let hash = compute_plugin_hash(&content);
    let coin: CoinDefinition = serde_json::from_str(&content)?;
    let validation = validate_plugin(&coin, &hash);
    
    if !validation.valid {
        return Err(CoreError::PluginValidation(validation.errors.join("; ")));
    }
    
    Ok((coin, validation))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blocked_url_schemes() {
        let coin = CoinDefinition {
            schema_version: 1,
            id: "test".to_string(),
            name: "Test".to_string(),
            symbol: "TST".to_string(),
            algorithm: "randomx".to_string(),
            recommended_miner: "xmrig".to_string(),
            cpu_mineable: true,
            default_pools: vec![crate::PoolConfig {
                name: "Bad Pool".to_string(),
                stratum_url: "file:///etc/passwd".to_string(),
                tls: false,
                region: "local".to_string(),
            }],
            notes: None,
            trusted: false,
        };

        let result = validate_plugin(&coin, "somehash");
        assert!(!result.valid);
        assert!(result.errors.iter().any(|e| e.contains("Invalid URL scheme")));
    }

    #[test]
    fn test_localhost_warning() {
        let coin = CoinDefinition {
            schema_version: 1,
            id: "test".to_string(),
            name: "Test".to_string(),
            symbol: "TST".to_string(),
            algorithm: "randomx".to_string(),
            recommended_miner: "xmrig".to_string(),
            cpu_mineable: true,
            default_pools: vec![crate::PoolConfig {
                name: "Local Pool".to_string(),
                stratum_url: "stratum+tcp://localhost:3333".to_string(),
                tls: false,
                region: "local".to_string(),
            }],
            notes: None,
            trusted: false,
        };

        let result = validate_plugin(&coin, "somehash");
        assert!(result.valid); // Valid but with warning
        assert!(result.warnings.iter().any(|w| w.contains("localhost")));
    }
}
