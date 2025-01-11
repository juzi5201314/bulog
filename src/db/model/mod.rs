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
    use smol_str::format_smolstr;

    use crate::db::model::{
        config::{ConfigRecordOption, query_config, update_config, verify_password},
        post::{
            PostRecord, PostRecordOption, create_post, query_post, query_posts_by_page, update_post,
        },
    };

    #[tokio::test]
    async fn test_db_config() -> anyhow::Result<()> {
        let db = crate::db::mem_db().await?;

        let conf = query_config(&db).await?;
        assert_eq!(conf.title, "bulog");

        update_config(&db, ConfigRecordOption {
            description: Some("updated".to_owned()),
            ..Default::default()
        })
        .await?;

        let conf = query_config(&db).await?;
        assert_eq!(conf.title, "bulog");
        assert_eq!(conf.description, "updated");

        Ok(())
    }

    #[tokio::test]
    async fn test_password() -> anyhow::Result<()> {
        let db = crate::db::mem_db().await?;
        let conf = query_config(&db).await?;
        assert!(conf.password.is_empty());
        assert!(verify_password(&db, "default".to_owned()).await?);
        update_config(&db, ConfigRecordOption {
            title: Some("new title".to_owned()),
            password: Some("pwd1".to_owned()),
            ..Default::default()
        })
        .await?;
        let conf = query_config(&db).await?;
        assert_eq!(conf.title, "new title");
        assert!(conf.password.is_empty());
        assert!(verify_password(&db, "pwd1".to_owned()).await?);
        assert!(!verify_password(&db, "pwd2".to_owned()).await?);
        Ok(())
    }

    #[tokio::test]
    async fn test_create_post() -> anyhow::Result<()> {
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

    #[tokio::test]
    async fn test_update_post() -> anyhow::Result<()> {
        let db = crate::db::mem_db().await?;
        let id = create_post(
            &db,
            "test title".into(),
            "test content".into(),
            false,
            false,
        )
        .await?;
        update_post(&db, id.clone(), PostRecordOption {
            title: Some("new title".into()),
            ..Default::default()
        })
        .await?;
        let post = query_post(&db, id).await?.unwrap();
        assert_eq!(post.title, "new title");
        Ok(())
    }

    #[tokio::test]
    async fn test_query_post() -> anyhow::Result<()> {
        let db = crate::db::mem_db().await?;
        for i in 0..101 {
            create_post(&db, format_smolstr!("post {i}"), "".into(), false, false).await?;
        }
        let posts = query_posts_by_page(&db, 0, 10, false).await?;
        assert_eq!(
            posts.get(0).map(|post| post.title.clone()),
            Some("post 100".into())
        );
        assert_eq!(
            posts.get(9).map(|post| post.title.clone()),
            Some("post 91".into())
        );
        assert_eq!(posts.len(), 10);
        let posts = query_posts_by_page(&db, 10, 10, false).await?;
        assert_eq!(posts.len(), 1);
        assert_eq!(
            posts.get(0).map(|post| post.title.clone()),
            Some("post 0".into())
        );
        Ok(())
    }
}
