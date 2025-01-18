use sqlx::prelude::FromRow;

#[derive(FromRow, serde::Serialize)]
pub struct MembershipDb {
    pub user_id: i32,
    pub channel_id: i32,
}
