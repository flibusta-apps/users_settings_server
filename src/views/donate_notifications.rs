use axum::{Router, response::IntoResponse, routing::{get, post}, extract::Path, Json, http::StatusCode};
use chrono::Duration;

use crate::{prisma::chat_donate_notifications, db::get_prisma_client};


async fn is_need_send(
    Path(chat_id): Path<i64>
) -> impl IntoResponse {
    const NOTIFICATION_DELTA_DAYS: i64 = 60;

    let client = get_prisma_client().await;

    let notification = client.chat_donate_notifications()
        .find_unique(chat_donate_notifications::chat_id::equals(chat_id))
        .exec()
        .await
        .unwrap();

    match notification {
        Some(notification) => {
            let now = chrono::offset::Local::now().naive_local();
            let check_date = now - Duration::days(NOTIFICATION_DELTA_DAYS);
            let result = notification.sended.naive_local() < check_date;

            Json(result).into_response()
        },
        None => Json(true).into_response(),
    }
}


async fn mark_sended(
    Path(chat_id): Path<i64>
) -> impl IntoResponse {
    let client = get_prisma_client().await;

    let _ = client.chat_donate_notifications()
        .upsert(
            chat_donate_notifications::chat_id::equals(chat_id),
            chat_donate_notifications::create(
                chat_id,
                chrono::offset::Local::now().into(),
                vec![]
            ),
            vec![
                chat_donate_notifications::sended::set(
                    chrono::offset::Local::now().into()
                )
            ]
        );

    StatusCode::OK
}


pub fn get_router() -> Router {
    Router::new()
        .route("/:chat_id/is_need_send", get(is_need_send))
        .route("/:chat_id", post(mark_sended))
}
