use tokio::signal;
use tracing::info;

pub async fn shutdown() {
    signal::ctrl_c().await.expect("Failed to install CTRL+C signal handler");
    info!("Shutdown signal received");
}