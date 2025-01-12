use salvo::{Router, handler};

use super::resp::Response;

mod auth;
mod config;
mod install;

pub fn router() -> Router {
    Router::with_path("v1")
        .push(install::router())
        .push(config::router())
        .push(auth::router())
}

#[handler]
pub async fn catch404() -> Response<()> {
    Response::custom(404, "not found")
}
