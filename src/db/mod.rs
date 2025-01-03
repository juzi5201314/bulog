use cfg_if::cfg_if;

fn select_endpoint() -> String {
    let _default = |_| {
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
    std::env::var("BU_ENDPOINT").unwrap_or_else(_default)
}

pub async fn db() -> anyhow::Result<()> {
    let db = surrealdb::engine::any::connect(select_endpoint()).await?;

    Ok(())
}
