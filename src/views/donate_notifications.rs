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

#[derive(sqlx::FromRow)]
struct ChatDonateNotification {
    pub sended: chrono::NaiveDateTime,
}

#[derive(Deserialize)]
struct IsNeedSendQuery {
    is_private: bool,
}

async fn is_need_send(
    Path(chat_id): Path<i64>,
    query: Query<IsNeedSendQuery>,
    db: Database,
) -> impl IntoResponse {
    const NOTIFICATION_DELTA_DAYS_PRIVATE: i64 = 60;
    const NOTIFICATION_DELTA_DAYS: i64 = 7;

    let notification = sqlx::query_as!(
        ChatDonateNotification,
        r#"SELECT sended FROM chat_donate_notifications WHERE chat_id = $1"#,
        chat_id
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    let delta_days = if query.is_private {
        NOTIFICATION_DELTA_DAYS_PRIVATE
    } else {
        NOTIFICATION_DELTA_DAYS
    };

    match notification {
        Some(notification) => {
            let result = notification.sended + Duration::days(delta_days)
                <= chrono::offset::Local::now().naive_local();
            Json(result).into_response()
        }
        None => Json(true).into_response(),
    }
}

async fn mark_sent(Path(chat_id): Path<i64>, db: Database) -> impl IntoResponse {
    sqlx::query_as!(
        ChatDonateNotification,
        r#"INSERT INTO chat_donate_notifications (chat_id, sended) VALUES ($1, $2)
        ON CONFLICT (chat_id) DO UPDATE SET sended = EXCLUDED.sended
        RETURNING sended"#,
        chat_id,
        chrono::offset::Local::now().naive_local()
    )
    .fetch_one(&db.0)
    .await
    .unwrap();

    StatusCode::OK
}

pub fn get_router() -> Router {
    Router::new()
        .route("/:chat_id/is_need_send", get(is_need_send))
        .route("/:chat_id", post(mark_sent))
}
