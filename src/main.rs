mod capture;
mod imaging;
mod server;

use anyhow::Result;
use rmcp::{ServiceExt, transport::stdio};
use tracing::info;

use server::ScreenshotServer;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "screenshot_mcp=info".parse().unwrap()),
        )
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    info!("Starting screenshot-mcp server");

    let service = ScreenshotServer::new().serve(stdio()).await?;
    service.waiting().await?;

    info!("Server shut down");
    Ok(())
}
