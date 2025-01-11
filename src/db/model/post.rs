use serde::{Deserialize, Deserializer, Serialize};
use smol_str::{SmolStr, ToSmolStr};
use surrealdb::{RecordId, Surreal, engine::any::Any};

use crate::nano_id::nanoid;

#[derive(Debug, Serialize, Deserialize, bulog_derive::Optional)]
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
            CREATE
                type::thing("post",$id)
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

pub async fn query_post(db: &Surreal<Any>, id: SmolStr) -> anyhow::Result<Option<PostRecord>> {
    db.select(("post", &*id)).await.map_err(Into::into)
}

pub async fn query_all_posts(db: &Surreal<Any>) -> anyhow::Result<Vec<PostRecord>> {
    db.select("post").await.map_err(Into::into)
}

pub async fn query_posts_by_page(
    db: &Surreal<Any>,
    page: usize,
    page_size: usize,
    asc: bool,
) -> anyhow::Result<Vec<PostRecord>> {
    // 因为surrealQL不允许用参数替代关键词, 所以只能出此下策用字符串拼接查询语句
    let mut resp = db
        .query(format!(
            "SELECT * FROM post ORDER BY created_time {order} LIMIT $limit START $start;",
            order = asc.then(|| "ASC").unwrap_or("DESC")
        ))
        .bind(("limit", page_size))
        .bind(("start", page * page_size))
        .await?;
    let posts: Vec<PostRecord> = resp.take(0)?;
    Ok(posts)
}

pub async fn delete_post(db: &Surreal<Any>, id: SmolStr) -> anyhow::Result<()> {
    db.delete(("post", &*id))
        .await
        .map_err(Into::into)
        .map(|_: Option<PostRecord>| ())
}

pub async fn update_post(
    db: &Surreal<Any>,
    id: SmolStr,
    mut post: PostRecordOption,
) -> anyhow::Result<()> {
    // 暂时不允许更改文章id
    post.id = None;
    db.update(("post", &*id))
        .merge(post)
        .await
        .map_err(Into::into)
        .map(|_: Option<PostRecord>| ())
}
