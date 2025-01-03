use serde::{Deserialize, Deserializer, Serialize};
use smol_str::{SmolStr, ToSmolStr};
use surrealdb::{RecordId, Surreal, engine::any::Any};

use crate::nano_id::nanoid;

#[derive(Debug, Serialize, Deserialize)]
pub struct PostRecord {
    pub title: SmolStr,
    pub content: SmolStr,
    pub created_time: surrealdb::Datetime,
    #[serde(deserialize_with = "deserialize_record_id")]
    pub id: SmolStr,
    pub draft: bool,
    pub pinned: bool,
}

fn deserialize_record_id<'de, D>(deserializer: D) -> Result<SmolStr, D::Error>
where
    D: Deserializer<'de>,
{
    let record_id = RecordId::deserialize(deserializer)?;
    Ok(record_id.key().to_smolstr())
}

pub async fn create_post(
    db: &Surreal<Any>,
    title: SmolStr,
    content: SmolStr,
    draft: bool,
    pinned: bool,
) -> anyhow::Result<SmolStr> {
    loop {
        let id = nanoid(6);
        let mut resp = db
            .query(
                r#"
        BEGIN TRANSACTION;

        IF record::exists(type::thing("post", $id)) {
            RETURN 0;
        } ELSE {
            CREATE type::thing("post",$id)
                SET 
                created_time = time::now(), 
                title = $title, 
                content = $content, 
                draft = $draft, 
                pinned = $pinned;
            RETURN 1;
        };

        COMMIT TRANSACTION;
    "#,
            )
            .bind(("id", id.clone()))
            .bind(("title", title.clone()))
            .bind(("content", content.clone()))
            .bind(("draft", draft))
            .bind(("pinned", pinned))
            .await?;
        let is_ok: Option<usize> = resp.take(0)?;

        if is_ok.is_some_and(|ret| ret == 1) {
            break Ok(id);
        }
    }
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
