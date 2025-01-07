use std::convert::identity;

use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::any::Any};

#[derive(Debug, Serialize, Deserialize, bulog_derive::Optional)]
pub struct ConfigRecord {
    pub title: String,
    pub description: String,
}

impl Default for ConfigRecord {
    fn default() -> Self {
        Self {
            title: "bulog".to_owned(),
            description: "A sample blog program".to_owned(),
        }
    }
}

pub async fn query_config(db: &Surreal<Any>) -> anyhow::Result<ConfigRecord> {
    db.select(("config", "bulog"))
        .await
        .map_err(Into::into)
        .map(|opt| opt.ok_or_else(|| anyhow::anyhow!("Uninitialized blog info")))
        .and_then(identity)
}

pub async fn update_config(db: &Surreal<Any>, config: ConfigRecordOption) -> anyhow::Result<()> {
    db.update(("config", "bulog"))
        .merge(config)
        .await
        .map_err(Into::into)
        .map(|_: Option<ConfigRecord>| ())
}
