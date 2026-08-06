use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Duration;
use serde::Deserialize;

use super::Database;
use crate::error::AppError;

const NOTIFICATION_DELTA_DAYS_PRIVATE: i64 = 60;
const NOTIFICATION_DELTA_DAYS: i64 = 7;

#[derive(Deserialize)]
struct IsNeedSendQuery {
    is_private: bool,
}

pub async fn is_need_send_check(
    pool: &sqlx::PgPool,
    chat_id: i64,
    is_private: bool,
) -> Result<bool, sqlx::Error> {
    let notification = sqlx::query!(
        r#"SELECT sended FROM chat_donate_notifications WHERE chat_id = $1"#,
        chat_id
    )
    .fetch_optional(pool)
    .await?;

    let delta_days = if is_private {
        NOTIFICATION_DELTA_DAYS_PRIVATE
    } else {
        NOTIFICATION_DELTA_DAYS
    };

    Ok(match notification {
        Some(notification) => {
            notification.sended + Duration::days(delta_days) <= chrono::Utc::now()
        }
        None => true,
    })
}

pub async fn try_claim(
    pool: &sqlx::PgPool,
    chat_id: i64,
    is_private: bool,
) -> Result<bool, sqlx::Error> {
    let delta_days = if is_private {
        NOTIFICATION_DELTA_DAYS_PRIVATE
    } else {
        NOTIFICATION_DELTA_DAYS
    };

    let now = chrono::Utc::now();
    let threshold = now - Duration::days(delta_days);

    let row = sqlx::query!(
        r#"
        INSERT INTO chat_donate_notifications (chat_id, sended)
        VALUES ($1, $2)
        ON CONFLICT (chat_id) DO UPDATE
        SET sended = $2
        WHERE chat_donate_notifications.sended <= $3
        RETURNING chat_id
        "#,
        chat_id,
        now,
        threshold
    )
    .fetch_optional(pool)
    .await?;

    Ok(row.is_some())
}

async fn is_need_send(
    Path(chat_id): Path<i64>,
    query: Query<IsNeedSendQuery>,
    db: Database,
) -> Result<impl IntoResponse, AppError> {
    let result = is_need_send_check(&db.0, chat_id, query.is_private).await?;

    Ok(Json(result).into_response())
}

async fn try_claim_handler(
    Path(chat_id): Path<i64>,
    query: Query<IsNeedSendQuery>,
    db: Database,
) -> Result<impl IntoResponse, AppError> {
    let result = try_claim(&db.0, chat_id, query.is_private).await?;

    Ok(Json(result).into_response())
}

async fn mark_sent(Path(chat_id): Path<i64>, db: Database) -> Result<impl IntoResponse, AppError> {
    sqlx::query!(
        r#"INSERT INTO chat_donate_notifications (chat_id, sended) VALUES ($1, $2)
        ON CONFLICT (chat_id) DO UPDATE SET sended = EXCLUDED.sended"#,
        chat_id,
        chrono::Utc::now()
    )
    .execute(&db.0)
    .await?;

    Ok(StatusCode::OK)
}

pub fn get_router() -> Router {
    Router::new()
        .route("/{chat_id}/is_need_send", get(is_need_send))
        .route("/{chat_id}/try_claim", post(try_claim_handler))
        .route("/{chat_id}", post(mark_sent))
}
