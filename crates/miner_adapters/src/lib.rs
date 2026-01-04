mod cpuminer_opt;
mod xmrig;

#[cfg(any(test, feature = "test-miners"))]
mod fake;

// Re-export common types from xmrig (canonical definitions)
pub use xmrig::{MinerState, MiningConfig, PerformancePreset};

// Re-export adapters
pub use cpuminer_opt::{
    CpuminerOptAdapter, CpuminerOptStats, 
    map_algorithm as cpuminer_map_algorithm,
    supports_algorithm as cpuminer_supports_algorithm,
    SUPPORTED_ALGORITHMS as CPUMINER_SUPPORTED_ALGORITHMS,
};

#[cfg(any(test, feature = "test-miners"))]
pub use fake::{FakeMinerAdapter, FakeCpuminerAdapter};

pub use xmrig::{XMRigAdapter, XMRigStats, XMRigHashrate, XMRigResults, XMRigConnection, XMRigCpu};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum AdapterError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Binary not found: {0}")]
    BinaryNotFound(String),
    #[error("Checksum mismatch - binary may be tampered")]
    ChecksumMismatch,
    #[error("Binary quarantined by macOS: {0}")]
    Quarantined(String),
    #[error("Download failed: {0}")]
    DownloadFailed(String),
    #[error("Process error: {0}")]
    Process(String),
    #[error("Path traversal detected")]
    PathTraversal,
    #[error("Invalid file permissions")]
    InvalidPermissions,
}

pub type Result<T> = std::result::Result<T, AdapterError>;

/// Validate binary path for security
pub fn validate_binary_path(path: &std::path::Path) -> Result<()> {
    // Check for path traversal
    let canonical = path.canonicalize().map_err(|_| AdapterError::PathTraversal)?;
    
    // Ensure it's within expected directories
    let allowed_dirs = [
        dirs::data_local_dir().map(|d| d.join("openminedash")),
        dirs::home_dir(),
    ];
    
    let is_allowed = allowed_dirs.iter().any(|dir| {
        dir.as_ref().map_or(false, |d| canonical.starts_with(d))
    });
    
    if !is_allowed {
        return Err(AdapterError::PathTraversal);
    }

    // Check file permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let metadata = std::fs::metadata(&canonical)?;
        let mode = metadata.permissions().mode();
        
        // Warn if world-writable
        if mode & 0o002 != 0 {
            tracing::warn!("Binary is world-writable, this is a security risk");
        }
    }

    Ok(())
}
