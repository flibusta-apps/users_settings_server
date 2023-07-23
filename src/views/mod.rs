use axum::{Router, response::Response, http::{StatusCode, self, Request}, middleware::{Next, self}};
use tower_http::trace::{TraceLayer, self};
use tracing::Level;

use crate::config::CONFIG;

pub mod users;
pub mod pagination;
pub mod languages;
pub mod donate_notifications;


async fn auth<B>(req: Request<B>, next: Next<B>) -> Result<Response, StatusCode> {
    let auth_header = req.headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    if auth_header != CONFIG.api_key {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}


pub fn get_router() -> Router {
    Router::new()
        .nest("/users/", users::get_router())
        .nest("/languages/", languages::get_router())
        .nest("/donate_notifications/", donate_notifications::get_router())
        .layer(middleware::from_fn(auth))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new()
                    .level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new()
                    .level(Level::INFO)),
        )
}
