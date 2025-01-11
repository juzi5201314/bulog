use cfg_if::cfg_if;
use model::{
    Record,
    config::{ConfigRecord, create_config},
};
use surrealdb::{Surreal, engine::any::Any};

pub mod model;

fn select_endpoint() -> String {
    let _default = || {
        cfg_if! {
            if #[cfg(feature = "rocksdb_backend")] {
                "rocksdb://./db.rocks".to_owned()
            } else if #[cfg(feature = "surrealkv_backend")] {
                "surrealkv://./db.surreal".to_owned()
            } else {
                "mem://".to_owned()
            }
        }
    };
    std::env::var("BU_ENDPOINT").ok().unwrap_or_else(_default)
}

/// test only
#[allow(unused)]
#[doc(hidden)]
pub(crate) async fn test_db() -> anyhow::Result<Surreal<Any>> {
    let db = db(Some("mem://".to_owned())).await?;
    create_config(&db, ConfigRecord::default()).await?;
    Ok(db)
}

pub async fn db(specified: Option<String>) -> anyhow::Result<Surreal<Any>> {
    let endpoint = specified.unwrap_or_else(select_endpoint);

    if endpoint.starts_with("mem:") {
        tracing::warn!(
            "You are using an in-memory database, and will lose all data if you stop it!"
        );
    }

    let db = surrealdb::engine::any::connect(endpoint).await?;
    initialize_db(&db).await?;
    Ok(db)
}

async fn initialize_db(db: &Surreal<Any>) -> anyhow::Result<()> {
    tracing::info!("Initializing database");
    db.use_ns("bulog").use_db("blog").await?;
    /* let blog_config: Option<ConfigRecord> = db.select(("config", "bulog")).await?;
    // 如果config表是空的, 那么认定博客程序未初始化
    // 在config表中插入一条`唯一`的记录, 用于存放博客全局配置
    // 记录id硬编码为: bulog
    if blog_config.is_none() {
        tracing::info!("first start, initializing blog config");
        let _: Option<Record> = db
            .create(("config", "bulog"))
            .content(ConfigRecord::default())
            .await?;
    } */
    Ok(())
}
