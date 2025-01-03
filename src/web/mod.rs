use std::pin::Pin;

use salvo::{Listener, Router, Server, conn::TcpListener, server::ServerHandle};

pub async fn web_server(
    listen_shutdown_signal: impl Fn(ServerHandle) -> Pin<Box<dyn Future<Output = ()> + Send>>,
) {
    let bind = std::env::var("BU_BIND").unwrap_or_else(|_| "0.0.0.0:8686".to_owned());
    let router = Router::new();

    let listener = TcpListener::new(bind).bind().await;
    let server = Server::new(listener);
    tokio::spawn(listen_shutdown_signal(server.handle()));
    server.serve(router).await;
}
