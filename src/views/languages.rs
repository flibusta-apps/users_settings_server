use axum::{http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use once_cell::sync::Lazy;
use serde::Serialize;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

use super::Database;
use crate::error::{ApiPath, AppError};

#[derive(sqlx::FromRow, Serialize, Clone)]
pub struct LanguageDetail {
    pub id: i32,
    pub label: String,
    pub code: String,
}

const LANGUAGES_CACHE_TTL: Duration = Duration::from_secs(600); // 10 minutes

struct LanguagesCacheEntry {
    data: Arc<Vec<LanguageDetail>>,
    fetched_at: Instant,
}

static LANGUAGES_CACHE: Lazy<RwLock<Option<LanguagesCacheEntry>>> = Lazy::new(|| RwLock::new(None));

/// Returns the full language list, served from an in-process cache with a TTL.
/// Used by both `GET /languages/` and `POST /users/` (to avoid a trailing re-SELECT).
pub async fn get_cached_languages(
    pool: &sqlx::PgPool,
) -> Result<Arc<Vec<LanguageDetail>>, sqlx::Error> {
    {
        let guard = LANGUAGES_CACHE.read().await;
        if let Some(entry) = guard.as_ref() {
            if entry.fetched_at.elapsed() < LANGUAGES_CACHE_TTL {
                return Ok(entry.data.clone());
            }
        }
    }

    let languages = sqlx::query_as!(LanguageDetail, "SELECT id, label, code FROM languages")
        .fetch_all(pool)
        .await?;
    let data = Arc::new(languages);

    let mut guard = LANGUAGES_CACHE.write().await;
    *guard = Some(LanguagesCacheEntry {
        data: data.clone(),
        fetched_at: Instant::now(),
    });

    Ok(data)
}

async fn get_languages(db: Database) -> Result<impl IntoResponse, AppError> {
    let languages = get_cached_languages(&db.0).await?;

    Ok(Json(languages.as_ref()).into_response())
}

async fn get_language_by_code(
    ApiPath(code): ApiPath<String>,
    db: Database,
) -> Result<impl IntoResponse, AppError> {
    let languages = get_cached_languages(&db.0).await?;
    let language = languages.iter().find(|l| l.code == code);

    Ok(match language {
        Some(v) => Json(v.clone()).into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    })
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get_languages))
        .route("/{code}", get(get_language_by_code))
}
