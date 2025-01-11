use std::convert::identity;

use serde::{Deserialize, Serialize};
use surrealdb::{Surreal, engine::any::Any};

#[derive(Debug, Serialize, Deserialize, bulog_derive::Optional)]
pub struct ConfigRecord {
    pub title: String,
    pub description: String,
    /// 确保无法获取到password
    #[serde(skip_deserializing)]
    pub password: String,
}

impl Default for ConfigRecord {
    fn default() -> Self {
        Self {
            title: "bulog".to_owned(),
            description: "A sample blog program".to_owned(),
            password: "".to_owned(),
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

pub async fn update_config(
    db: &Surreal<Any>,
    mut config: ConfigRecordOption,
) -> anyhow::Result<()> {
    if let Some(pwd) = &config.password {
        update_password(db, pwd.clone()).await?;
        config.password = None;
    }
    db.update(("config", "bulog"))
        .merge(config)
        .await
        .map_err(Into::into)
        .map(|_: Option<ConfigRecord>| ())
}

pub async fn update_password(db: &Surreal<Any>, pwd: String) -> anyhow::Result<()> {
    db.query("UPDATE config:bulog SET password = crypto::argon2::generate($pwd)")
        .bind(("pwd", pwd))
        .await
        .map_err(Into::into)
        .map(|_| ())
}

pub async fn verify_password(db: &Surreal<Any>, pwd: String) -> anyhow::Result<bool> {
    db.query("RETURN crypto::argon2::compare((SELECT password FROM ONLY config:bulog).password, $pwd)")
        .bind(("pwd", pwd))
        .await
        .and_then(|mut res| res.take::<Option<_>>(0))
        .map(|opt| opt.unwrap_or_default())
        .map_err(Into::into)
}
