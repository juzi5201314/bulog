use serde::Deserialize;
use surrealdb::RecordId;

pub mod config;
pub mod post;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: RecordId,
}

#[cfg(test)]
mod tests {
    use crate::db::model::{
        config::{ConfigRecordOption, query_config, update_config},
        post::{PostRecord, create_post},
    };

    #[tokio::test]
    async fn db_config() -> anyhow::Result<()> {
        let db = crate::db::mem_db().await?;

        let conf = query_config(&db).await?;
        assert_eq!(conf.title, "bulog");
        assert_eq!(conf.description, "A sample blog program");

        update_config(&db, ConfigRecordOption {
            description: Some("updated".to_owned()),
            //title: Some("bulog".to_owned()),
            ..Default::default()
        })
        .await?;

        let conf = query_config(&db).await?;
        assert_eq!(conf.title, "bulog");
        assert_eq!(conf.description, "updated");

        Ok(())
    }

    #[tokio::test]
    async fn test_db_create_post() -> anyhow::Result<()> {
        let db = crate::db::mem_db().await?;

        let id = create_post(
            &db,
            "test title".into(),
            "test content".into(),
            false,
            false,
        )
        .await?;

        let posts: Vec<PostRecord> = db.query("select * from post").await?.take(0)?;

        assert_eq!(posts[0].id, id);
        assert_eq!(posts[0].title, "test title");
        assert_eq!(posts[0].content, "test content");

        Ok(())
    }
}
