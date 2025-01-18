use sqlx::prelude::FromRow;

#[derive(FromRow, serde::Serialize)]
pub struct UserFromDB {
    pub id: i32,
    pub username: String,
}

#[derive(FromRow, serde::Serialize)]
pub struct UserFromDBWithPassword {
    pub id: i32,
    pub username: String,
    pub password: String,
}
