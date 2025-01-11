use salvo::{Depot, Router, handler};
use surrealdb::{Surreal, engine::any::Any};

use crate::{
    db::model::config::{ConfigRecord, query_config},
    web::resp::{RespResult, Response},
};

pub fn router() -> Router {
    Router::with_path("config").get(get_config)
}

#[handler]
async fn get_config(depot: &mut Depot) -> RespResult<ConfigRecord> {
    let db = depot.obtain::<Surreal<Any>>().unwrap();
    query_config(db)
        .await
        .map(|c| Response::ok(c))
        .map_err(Into::into)
}
