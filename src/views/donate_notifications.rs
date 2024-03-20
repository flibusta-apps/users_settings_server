use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::Duration;
use serde::Deserialize;

use crate::prisma::chat_donate_notifications;

use super::Database;

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

    let notification = db
        .chat_donate_notifications()
        .find_unique(chat_donate_notifications::chat_id::equals(chat_id))
        .exec()
        .await
        .unwrap();

    let delta_days = if query.is_private {
        NOTIFICATION_DELTA_DAYS_PRIVATE
    } else {
        NOTIFICATION_DELTA_DAYS
    };

    match notification {
        Some(notification) => {
            let result = notification.sended.naive_local() + Duration::days(delta_days)
                <= chrono::offset::Local::now().naive_local();
            Json(result).into_response()
        }
        None => Json(true).into_response(),
    }
}

async fn mark_sent(Path(chat_id): Path<i64>, db: Database) -> impl IntoResponse {
    let _ = db
        .chat_donate_notifications()
        .upsert(
            chat_donate_notifications::chat_id::equals(chat_id),
            chat_donate_notifications::create(chat_id, chrono::offset::Local::now().into(), vec![]),
            vec![chat_donate_notifications::sended::set(
                chrono::offset::Local::now().into(),
            )],
        )
        .exec()
        .await;

    StatusCode::OK
}

pub fn get_router() -> Router {
    Router::new()
        .route("/:chat_id/is_need_send", get(is_need_send))
        .route("/:chat_id", post(mark_sent))
}
