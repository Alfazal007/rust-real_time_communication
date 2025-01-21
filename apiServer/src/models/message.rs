use sqlx::prelude::FromRow;

#[derive(FromRow, serde::Serialize)]
pub struct MessagesDb {
    pub id: i32,
    pub sender_id: i32,
    pub channel_id: i32,
}
