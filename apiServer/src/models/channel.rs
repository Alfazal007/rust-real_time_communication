use sqlx::prelude::FromRow;

#[derive(FromRow, serde::Serialize)]
pub struct ChannelDB {
    pub id: i32,
    pub name: String,
}
