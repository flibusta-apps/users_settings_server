use axum::{Router, response::IntoResponse, routing::get, Json, extract::Path, http::StatusCode};
use serde::Serialize;

use crate::{prisma::language, db::get_prisma_client};


#[derive(Serialize)]
pub struct LanguageDetail {
    pub id: i32,
    pub label: String,
    pub code: String,
}


impl From<language::Data> for LanguageDetail {
    fn from(value: language::Data) -> Self {
        let language::Data { id, label, code, .. } = value;

        Self {
            id,
            label,
            code
        }
    }
}


async fn get_languages() -> impl IntoResponse {
    let client = get_prisma_client().await;

    let languages: Vec<LanguageDetail> = client.language()
        .find_many(vec![])
        .exec()
        .await
        .unwrap()
        .into_iter()
        .map(|item| item.into())
        .collect();

    Json(languages).into_response()
}


async fn get_language_by_code(
    Path(code): Path<String>
) -> impl IntoResponse {
    let client = get_prisma_client().await;

    let language = client.language()
        .find_unique(language::code::equals(code))
        .exec()
        .await
        .unwrap();

    match language {
        Some(v) => Json::<LanguageDetail>(v.into()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}


pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get_languages))
        .route("/:code", get(get_language_by_code))
}
