use tokio::process::Child;
use tokio::time::{timeout, Duration};
use tracing::{info, warn};

/// Gracefully stop a child process with SIGTERM, falling back to SIGKILL
pub async fn graceful_stop(child: &mut Child) {
    info!("Stopping miner process");

    // Try SIGTERM first
    #[cfg(unix)]
    {
        use nix::sys::signal::{kill, Signal};
        use nix::unistd::Pid;

        if let Some(pid) = child.id() {
            let _ = kill(Pid::from_raw(pid as i32), Signal::SIGTERM);
        }
    }

    // Wait up to 3 seconds for graceful shutdown
    match timeout(Duration::from_secs(3), child.wait()).await {
        Ok(Ok(status)) => {
            info!("Miner stopped gracefully with status: {}", status);
        }
        Ok(Err(e)) => {
            warn!("Error waiting for miner: {}", e);
        }
        Err(_) => {
            warn!("Miner did not stop gracefully, sending SIGKILL");
            let _ = child.kill().await;
        }
    }
}
