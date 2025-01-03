use std::time::Duration;

use salvo::server::ServerHandle;
use tokio::signal;
use web::web_server;

mod db;
mod web;

mod nano_id;

fn main() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt().init();
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime build failed")
        .block_on(async_main());
}

async fn async_main() {
    tokio::spawn(web_server(|h| Box::pin(listen_shutdown_signal(h))));
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
        _ = ctrl_c => println!("ctrl_c signal received"),
        _ = terminate => println!("terminate signal received"),
    };

    // Graceful Shutdown Server
    server_handle.stop_graceful(Some(Duration::from_secs(5)));
}
