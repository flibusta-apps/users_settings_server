use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, sqlx::Type, Serialize)]
#[sqlx(type_name = "user_language_type")]
pub struct UserLanguage {
    pub id: i32,
    pub label: String,
    pub code: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct SimpleUser {
    pub id: i32,
    pub user_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub username: String,
    pub source: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct UserDetail {
    pub id: i32,
    pub user_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub username: String,
    pub source: String,
    pub allowed_langs: Vec<UserLanguage>,
}

#[derive(Deserialize)]
pub struct CreateOrUpdateUserData {
    pub user_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub username: String,
    pub source: String,
    pub allowed_langs: Vec<String>,
}
