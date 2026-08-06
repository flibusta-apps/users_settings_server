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
    pub default_search: Option<String>,
    pub file_name_lang: String,
}

#[derive(sqlx::FromRow, Serialize)]
pub struct UserDetail {
    pub id: i32,
    pub user_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub username: String,
    pub source: String,
    pub default_search: Option<String>,
    pub file_name_lang: String,
    pub allowed_langs: Vec<UserLanguage>,
}

#[derive(Deserialize)]
pub struct CreateOrUpdateUserData {
    pub user_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub username: String,
    pub source: String,
    pub default_search: Option<String>,
    pub file_name_lang: Option<String>,
    pub allowed_langs: Vec<String>,
}

const ALLOWED_DEFAULT_SEARCH: [&str; 4] = ["book", "author", "series", "translator"];
const ALLOWED_FILE_NAME_LANG: [&str; 2] = ["normalized", "original"];

impl CreateOrUpdateUserData {
    pub fn validate(&self) -> Result<(), String> {
        if self.last_name.chars().count() > 64 {
            return Err("last_name must be at most 64 characters".to_string());
        }
        if self.first_name.chars().count() > 64 {
            return Err("first_name must be at most 64 characters".to_string());
        }
        if self.username.chars().count() > 32 {
            return Err("username must be at most 32 characters".to_string());
        }
        if self.source.chars().count() > 32 {
            return Err("source must be at most 32 characters".to_string());
        }

        if let Some(default_search) = &self.default_search {
            if !ALLOWED_DEFAULT_SEARCH.contains(&default_search.as_str()) {
                return Err(format!(
                    "default_search must be one of: {}",
                    ALLOWED_DEFAULT_SEARCH.join(", ")
                ));
            }
        }

        if let Some(file_name_lang) = &self.file_name_lang {
            if !ALLOWED_FILE_NAME_LANG.contains(&file_name_lang.as_str()) {
                return Err(format!(
                    "file_name_lang must be one of: {}",
                    ALLOWED_FILE_NAME_LANG.join(", ")
                ));
            }
        }

        Ok(())
    }
}
