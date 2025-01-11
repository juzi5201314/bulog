use std::time::Duration;

use salvo::server::ServerHandle;
use tokio::signal;
use web::web_server;

mod db;
mod web;

mod nano_id;

fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("BU_LOG").unwrap_or_else(|_| "warn,bulog=info".to_owned()))
        .init();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime build failed")
        .block_on(async_main());
}

async fn async_main() {
    let (server_handle, join_handle) = web_server().await.unwrap();

    listen_shutdown_signal(server_handle).await;
    if let Err(_) = tokio::time::timeout(Duration::from_millis(3500), join_handle).await {
        tracing::warn!("shutdown server timeout, force termination");
    } else {
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
}

async fn listen_shutdown_signal(server_handle: ServerHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(windows)]
    let terminate = async {
        signal::windows::ctrl_c()
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => tracing::info!("ctrl_c signal received"),
        _ = terminate => tracing::info!("terminate signal received"),
    };

    tracing::info!("shutting down the server...");

    // Graceful Shutdown Server
    server_handle.stop_graceful(Some(Duration::from_secs(3)));
}
