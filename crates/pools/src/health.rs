use crate::{PoolError, Result};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;
use tracing::{info, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PoolStatus {
    Ok,
    Degraded,
    Down,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoolHealthResult {
    pub url: String,
    pub status: PoolStatus,
    pub connected: bool,
    pub tls_verified: Option<bool>,
    pub latency_ms: Option<u64>,
    pub error: Option<String>,
}

/// Comprehensive pool health check with TCP + optional TLS + stratum probe
pub async fn check_health(url: &str) -> Result<PoolHealthResult> {
    let (host, port, use_tls) = parse_stratum_url(url)?;

    info!("Checking pool health: {}:{} (TLS: {})", host, port, use_tls);

    let start = Instant::now();

    // Step 1: TCP connect
    let stream = match timeout(
        Duration::from_secs(5),
        TcpStream::connect((host.as_str(), port)),
    )
    .await
    {
        Ok(Ok(s)) => s,
        Ok(Err(e)) => {
            return Ok(PoolHealthResult {
                url: url.to_string(),
                status: PoolStatus::Down,
                connected: false,
                tls_verified: None,
                latency_ms: None,
                error: Some(format!("TCP connect failed: {}", e)),
            });
        }
        Err(_) => {
            return Ok(PoolHealthResult {
                url: url.to_string(),
                status: PoolStatus::Down,
                connected: false,
                tls_verified: None,
                latency_ms: None,
                error: Some("Connection timeout (5s)".to_string()),
            });
        }
    };

    let tcp_latency = start.elapsed().as_millis() as u64;

    // Step 2: TLS handshake if needed
    let tls_verified = if use_tls {
        match try_tls_handshake(stream, &host).await {
            Ok(_) => Some(true),
            Err(e) => {
                warn!("TLS handshake failed: {}", e);
                return Ok(PoolHealthResult {
                    url: url.to_string(),
                    status: PoolStatus::Degraded,
                    connected: true,
                    tls_verified: Some(false),
                    latency_ms: Some(tcp_latency),
                    error: Some(format!("TLS handshake failed: {}", e)),
                });
            }
        }
    } else {
        // For non-TLS, try a basic stratum probe
        match try_stratum_probe(stream).await {
            Ok(_) => None,
            Err(e) => {
                warn!("Stratum probe failed: {}", e);
                // Still connected, just degraded
                return Ok(PoolHealthResult {
                    url: url.to_string(),
                    status: PoolStatus::Degraded,
                    connected: true,
                    tls_verified: None,
                    latency_ms: Some(tcp_latency),
                    error: Some(format!("Stratum probe failed: {}", e)),
                });
            }
        }
    };

    let total_latency = start.elapsed().as_millis() as u64;

    // Determine status based on latency
    let status = if total_latency > 500 {
        PoolStatus::Degraded
    } else {
        PoolStatus::Ok
    };

    Ok(PoolHealthResult {
        url: url.to_string(),
        status,
        connected: true,
        tls_verified,
        latency_ms: Some(total_latency),
        error: None,
    })
}

async fn try_tls_handshake(stream: TcpStream, host: &str) -> Result<()> {
    use tokio_native_tls::TlsConnector;

    let connector = native_tls::TlsConnector::new()
        .map_err(|e| PoolError::ConnectionFailed(e.to_string()))?;
    let connector = TlsConnector::from(connector);

    timeout(Duration::from_secs(5), connector.connect(host, stream))
        .await
        .map_err(|_| PoolError::Timeout)?
        .map_err(|e| PoolError::ConnectionFailed(format!("TLS error: {}", e)))?;

    Ok(())
}

async fn try_stratum_probe(mut stream: TcpStream) -> Result<()> {
    // Send a minimal stratum subscribe request
    let subscribe = r#"{"id":1,"method":"mining.subscribe","params":[]}"#;
    let msg = format!("{}\n", subscribe);

    timeout(Duration::from_secs(3), stream.write_all(msg.as_bytes()))
        .await
        .map_err(|_| PoolError::Timeout)?
        .map_err(|e| PoolError::Io(e))?;

    // Try to read response (we don't need to parse it, just verify we get something)
    let mut buf = [0u8; 1024];
    match timeout(Duration::from_secs(3), stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => Ok(()),
        Ok(Ok(_)) => Err(PoolError::ConnectionFailed("Empty response".to_string())),
        Ok(Err(e)) => Err(PoolError::Io(e)),
        Err(_) => Err(PoolError::Timeout),
    }
}

fn parse_stratum_url(url: &str) -> Result<(String, u16, bool)> {
    let use_tls = url.contains("+ssl") || url.contains("+tls");

    let cleaned = url
        .trim_start_matches("stratum+tcp://")
        .trim_start_matches("stratum+ssl://")
        .trim_start_matches("stratum+tls://")
        .trim_start_matches("stratum://");

    let parts: Vec<&str> = cleaned.split(':').collect();
    if parts.len() != 2 {
        return Err(PoolError::InvalidUrl(format!(
            "Invalid stratum URL format: {}",
            url
        )));
    }

    let host = parts[0].to_string();
    let port = parts[1]
        .parse::<u16>()
        .map_err(|_| PoolError::InvalidUrl(format!("Invalid port in URL: {}", url)))?;

    Ok((host, port, use_tls))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_stratum_url() {
        let (host, port, tls) = parse_stratum_url("stratum+tcp://pool.example.com:3333").unwrap();
        assert_eq!(host, "pool.example.com");
        assert_eq!(port, 3333);
        assert!(!tls);

        let (host, port, tls) = parse_stratum_url("stratum+ssl://pool.example.com:14433").unwrap();
        assert_eq!(host, "pool.example.com");
        assert_eq!(port, 14433);
        assert!(tls);
    }
}
