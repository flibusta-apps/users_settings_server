use serde::{Serialize, Deserialize};

use crate::prisma::{user_settings, language};


#[derive(Serialize)]
pub struct UserLanguage {
    pub id: i32,
    pub label: String,
    pub code: String,
}


impl From<language::Data> for UserLanguage {
    fn from(value: language::Data) -> Self {
        Self {
            id: value.id,
            label: value.label,
            code: value.code
        }
    }
}


#[derive(Serialize)]
pub struct UserDetail {
    pub id: i32,
    pub user_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub username: String,
    pub source: String,
    pub allowed_langs: Vec<UserLanguage>
}


impl From<user_settings::Data> for UserDetail {
    fn from(value: user_settings::Data) -> Self {
        let allowed_langs: Vec<UserLanguage> = value
            .languages.unwrap()
            .into_iter()
            .map(|item| *item.language.unwrap())
            .map(|item| item.into())
            .collect();

        Self {
            id: value.id,
            user_id: value.user_id,
            last_name: value.last_name,
            first_name: value.first_name,
            username: value.username,
            source: value.source,
            allowed_langs
        }
    }
}

#[derive(Deserialize)]
pub struct CreateOrUpdateUserData {
    pub user_id: i64,
    pub last_name: String,
    pub first_name: String,
    pub username: String,
    pub source: String,
    pub allowed_langs: Vec<String>
}
