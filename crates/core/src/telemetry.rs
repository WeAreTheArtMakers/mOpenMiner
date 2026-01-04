use serde::{Deserialize, Serialize};

/// Stats from XMRig API
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct XMRigStats {
    pub hashrate: XMRigHashrate,
    pub results: XMRigResults,
    pub connection: XMRigConnection,
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
