use salvo::{
    Depot, Router, Writer, handler,
    session::{Session, SessionDepotExt},
};
use serde::Deserialize;

use crate::{
    db::model::config::verify_password,
    web::{
        extractors::{Json, logged},
        resp::{RespResult, Response},
    },
};

pub fn router() -> Router {
    Router::new()
        .path("login")
        .get(is_logged)
        .post(login)
}

#[derive(Deserialize)]
pub struct LoginPost {
    pub password: String,
}

#[handler]
async fn login(json: Json<LoginPost>, depot: &mut Depot) -> RespResult<()> {
    if logged(depot) {
        return Ok(Response::empty());
    }

    let Json(json) = json;
    let db = depot.obtain().unwrap();

    if verify_password(db, json.password).await? {
        let mut session = Session::new();
        session.insert("logged", true)?;
        depot.set_session(session);
        Ok(Response::empty())
    } else {
        Err(Response::custom(401, "login failure"))
    }
}

#[handler]
async fn is_logged(depot: &mut Depot) -> RespResult<()> {
    if logged(depot) {
        Ok(Response::empty())
    } else {
        Err(Response::custom(403, "not logged"))
    }
}
