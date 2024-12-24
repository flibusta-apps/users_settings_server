use axum::{extract::Path, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use serde::Serialize;

use super::Database;

#[derive(sqlx::FromRow, Serialize)]
pub struct LanguageDetail {
    pub id: i32,
    pub label: String,
    pub code: String,
}

async fn get_languages(db: Database) -> impl IntoResponse {
    let languages = sqlx::query_as!(LanguageDetail, "SELECT id, label, code FROM languages")
        .fetch_all(&db.0)
        .await
        .unwrap();

    Json(languages).into_response()
}

async fn get_language_by_code(Path(code): Path<String>, db: Database) -> impl IntoResponse {
    let language = sqlx::query_as!(
        LanguageDetail,
        r#"SELECT id, label, code FROM languages WHERE code = $1"#,
        code
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    match language {
        Some(v) => Json(v).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get_languages))
        .route("/:code", get(get_language_by_code))
}
