use salvo::{Depot, Router, Writer, handler};
use surrealdb::{Surreal, engine::any::Any};

use crate::db::model::config::{create_config, ConfigRecord};
use crate::web::Installed;
use crate::web::extractors::Json;
use crate::web::resp::{RespResult, Response};

pub fn router() -> Router {
    Router::with_path("install").post(install)
}

#[handler]
async fn install(config: Json<ConfigRecord>, depot: &mut Depot) -> RespResult<()> {
    let Json(config) = config;
    let db = depot.obtain::<Surreal<Any>>().unwrap();
    if depot.contains::<Installed>() {
        return Err(Response::error("repeat installation"));
    }

    create_config(db, config).await?;
    Ok(Response::empty())
}
