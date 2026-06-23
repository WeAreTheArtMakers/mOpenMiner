//! Algorithm routing for selecting the appropriate miner.
//!
//! This module determines which miner adapter to use based on:
//! 1. The coin's algorithm
//! 2. Whether "Try Mining Anyway" mode is enabled
//! 3. Miner availability and algorithm support

use serde::{Deserialize, Serialize};

/// XMRig supported algorithms (RandomX family + CryptoNight + Argon2)
/// Reference: https://xmrig.com/docs/algorithms
pub const XMRIG_ALGORITHMS: &[&str] = &[
    // RandomX family
    "randomx",
    "rx/0",
    "rx/wow",
    "rx/arq",
    "rx/sfx",
    "rx/keva",
    // CryptoNight family
    "cn/0",
    "cn/1",
    "cn/2",
    "cn/r",
    "cn/fast",
    "cn/half",
    "cn/xao",
    "cn/rto",
    "cn/rwz",
    "cn/zls",
    "cn/double",
    "cn-lite/0",
    "cn-lite/1",
    "cn-heavy/0",
    "cn-heavy/tube",
    "cn-heavy/xhv",
    "cn-pico",
    "cn-pico/tlo",
    "cn/ccx",
    "cn/upx2",
    // Argon2
    "argon2/chukwa",
    "argon2/chukwav2",
    "argon2/ninja",
    "argon2/wrkz",
    // GhostRider
    "ghostrider",
    "gr",
    // VerusHash
    "verushash",
    "verushash/2",
    "verushash/2.1",
];

/// cpuminer-opt supported algorithms
/// Reference: https://github.com/JayDDee/cpuminer-opt
pub const CPUMINER_OPT_ALGORITHMS: &[&str] = &[
    "sha256d",
    "sha256",
    "scrypt",
    "x11",
    "x13",
    "x14",
    "x15",
    "x16r",
    "x16rv2",
    "x16s",
    "x17",
    "x21s",
    "x22i",
    "x25x",
    "lyra2v2",
    "lyra2v3",
    "lyra2z",
    "lyra2h",
    "yescrypt",
    "yescryptr8",
    "yescryptr16",
    "yescryptr32",
    "yespower",
    "yespowerr16",
    "allium",
    "blake",
    "blake2b",
    "blake2s",
    "groestl",
    "heavy",
    "keccak",
    "lbry",
    "neoscrypt",
    "nist5",
    "phi1612",
    "phi2",
    "quark",
    "qubit",
    "skein",
    "skein2",
    "tribus",
    "whirlpool",
];

/// Algorithms that are GPU-only (no CPU miner support)
pub const GPU_ONLY_ALGORITHMS: &[&str] = &[
    "ethash",
    "etchash",
    "kawpow",
    "kheavyhash",
    "autolykos2",
    "equihash",
    "zelhash",
    "cuckoo",
    "cuckatoo",
    "cuckaroo",
    "beamhash",
    "progpow",
];

/// Algorithms that require ASIC hardware
pub const ASIC_ALGORITHMS: &[&str] = &[
    "sha256d",  // BTC - technically CPU mineable but impractical
    "scrypt",   // LTC/DOGE - technically CPU mineable but impractical
];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MinerType {
    XMRig,
    CpuminerOpt,
    External,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingResult {
    pub miner_type: MinerType,
    pub algorithm: String,
    pub warning: Option<String>,
    pub is_practical: bool,
}

/// Determine which miner to use for a given algorithm
pub fn route_algorithm(algo: &str, try_anyway: bool) -> RoutingResult {
    let algo_lower = algo.to_lowercase();
    
    // Check XMRig first (preferred for supported algos)
    if is_xmrig_supported(&algo_lower) {
        return RoutingResult {
            miner_type: MinerType::XMRig,
            algorithm: algo_lower,
            warning: None,
            is_practical: true,
        };
    }
    
    // Check if GPU-only
    if is_gpu_only(&algo_lower) {
        if try_anyway {
            // Check if cpuminer-opt has any support
            if is_cpuminer_supported(&algo_lower) {
                return RoutingResult {
                    miner_type: MinerType::CpuminerOpt,
                    algorithm: map_to_cpuminer_algo(&algo_lower),
                    warning: Some("This algorithm is optimized for GPU. CPU mining will be extremely slow.".to_string()),
                    is_practical: false,
                };
            }
        }
        return RoutingResult {
            miner_type: MinerType::External,
            algorithm: algo_lower,
            warning: Some("This algorithm requires GPU hardware.".to_string()),
            is_practical: false,
        };
    }
    
    // Check cpuminer-opt
    if is_cpuminer_supported(&algo_lower) {
        let is_asic = is_asic_algorithm(&algo_lower);
        return RoutingResult {
            miner_type: MinerType::CpuminerOpt,
            algorithm: map_to_cpuminer_algo(&algo_lower),
            warning: if is_asic {
                Some("This algorithm is dominated by ASIC miners. CPU hashrate will be negligible.".to_string())
            } else {
                None
            },
            is_practical: !is_asic,
        };
    }
    
    // Unsupported
    RoutingResult {
        miner_type: MinerType::Unsupported,
        algorithm: algo_lower,
        warning: Some("No CPU miner available for this algorithm.".to_string()),
        is_practical: false,
    }
}

fn is_xmrig_supported(algo: &str) -> bool {
    XMRIG_ALGORITHMS.iter().any(|a| {
        a.eq_ignore_ascii_case(algo) || 
        algo.starts_with(&format!("{}/", a.split('/').next().unwrap_or("")))
    })
}

fn is_cpuminer_supported(algo: &str) -> bool {
    CPUMINER_OPT_ALGORITHMS.iter().any(|a| a.eq_ignore_ascii_case(algo))
}

fn is_gpu_only(algo: &str) -> bool {
    GPU_ONLY_ALGORITHMS.iter().any(|a| a.eq_ignore_ascii_case(algo))
}

fn is_asic_algorithm(algo: &str) -> bool {
    ASIC_ALGORITHMS.iter().any(|a| a.eq_ignore_ascii_case(algo))
}

fn map_to_cpuminer_algo(algo: &str) -> String {
    match algo {
        "sha256" | "sha-256" => "sha256d".to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xmrig_routing() {
        let result = route_algorithm("randomx", false);
        assert_eq!(result.miner_type, MinerType::XMRig);
        assert!(result.is_practical);
    }

    #[test]
    fn test_btc_routing() {
        // Without try_anyway
        let result = route_algorithm("sha256d", false);
        assert_eq!(result.miner_type, MinerType::CpuminerOpt);
        assert!(!result.is_practical); // ASIC dominated
        
        // With try_anyway
        let result = route_algorithm("sha256d", true);
        assert_eq!(result.miner_type, MinerType::CpuminerOpt);
    }

    #[test]
    fn test_gpu_only_routing() {
        let result = route_algorithm("ethash", false);
        assert_eq!(result.miner_type, MinerType::External);
        
        let result = route_algorithm("kawpow", true);
        assert_eq!(result.miner_type, MinerType::External);
    }

    #[test]
    fn test_scrypt_routing() {
        let result = route_algorithm("scrypt", false);
        assert_eq!(result.miner_type, MinerType::CpuminerOpt);
        assert!(!result.is_practical); // ASIC dominated
    }

    #[test]
    fn test_verushash_routing() {
        let result = route_algorithm("verushash", false);
        assert_eq!(result.miner_type, MinerType::XMRig);
        assert!(result.is_practical);
    }

    #[test]
    fn test_ghostrider_routing() {
        let result = route_algorithm("ghostrider", false);
        assert_eq!(result.miner_type, MinerType::XMRig);
        assert!(result.is_practical);
    }
}
