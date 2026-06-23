mod api;
mod health;
mod stratum;

pub use api::*;
pub use health::*;
pub use stratum::*;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum PoolError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Timeout")]
    Timeout,
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, PoolError>;
