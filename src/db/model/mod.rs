use serde::Deserialize;
use surrealdb::RecordId;

pub mod config;
pub mod post;

#[derive(Debug, Deserialize)]
pub struct Record {
    pub id: RecordId,
}
