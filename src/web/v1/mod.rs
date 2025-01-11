use salvo::Router;

mod config;
mod install;
mod auth;

pub fn router() -> Router {
    Router::with_path("v1")
        .push(install::router())
        .push(config::router())
        .push(auth::router())
}
