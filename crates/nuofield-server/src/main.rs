use std::{env, net::SocketAddr, path::PathBuf};

use anyhow::{Context, Result};
use nuofield_server::{router, AppState};
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("nuofield_server=info,tower_http=info")),
        )
        .init();

    let bind = env::var("NUOFIELD_BIND").unwrap_or_else(|_| "127.0.0.1:3000".into());
    let bind: SocketAddr = bind
        .parse()
        .context("NUOFIELD_BIND must be a socket address")?;
    let data_dir = env::var_os("NUOFIELD_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("./data"));
    let state = AppState::load(data_dir.join("events.jsonl"))
        .context("failed to load the user-controlled event store")?;

    let listener = TcpListener::bind(bind)
        .await
        .with_context(|| format!("failed to bind {bind}"))?;
    info!(%bind, data_dir = %data_dir.display(), "NuoField server started");

    axum::serve(listener, router(state))
        .with_graceful_shutdown(shutdown_signal())
        .await
        .context("server failed")
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        tracing::error!(%error, "failed to install shutdown signal handler");
    }
}
