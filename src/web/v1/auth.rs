use salvo::{handler, session::Session, Depot, Router};

pub fn router() -> Router {
    Router::new().path("login").post(login).path("logout")
}

#[handler]
async fn login(depot: &mut Depot) {
    let mut session = Session::new();
}