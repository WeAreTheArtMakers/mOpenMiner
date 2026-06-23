use serde::{Deserialize, Serialize};

/// Stratum protocol message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratumRequest {
    pub id: u64,
    pub method: String,
    pub params: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratumResponse {
    pub id: u64,
    pub result: Option<serde_json::Value>,
    pub error: Option<StratumError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StratumError {
    pub code: i32,
    pub message: String,
}

/// Create a stratum login request
pub fn create_login_request(id: u64, wallet: &str, worker: &str) -> StratumRequest {
    StratumRequest {
        id,
        method: "login".to_string(),
        params: serde_json::json!({
            "login": wallet,
            "pass": worker,
            "agent": "OpenMineDash/0.1.0"
        }),
    }
}
