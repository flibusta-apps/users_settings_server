use axum::{
    http::{self, Request, StatusCode},
    middleware::{self, Next},
    response::Response,
    routing::get,
    Extension, Router,
};
use axum_prometheus::PrometheusMetricLayer;
use sqlx::PgPool;
use subtle::ConstantTimeEq;

use tower_http::catch_panic::CatchPanicLayer;
use tower_http::trace::{self, TraceLayer};
use tracing::Level;

use crate::{config::CONFIG, db::get_postgres_pool};

pub mod donate_notifications;
pub mod languages;
pub mod pagination;
pub mod users;

pub type Database = Extension<PgPool>;

async fn auth(req: Request<axum::body::Body>, next: Next) -> Result<Response, StatusCode> {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let auth_header = if let Some(auth_header) = auth_header {
        auth_header
    } else {
        return Err(StatusCode::UNAUTHORIZED);
    };

    let provided_key = auth_header
        .strip_prefix("Bearer ")
        .unwrap_or(auth_header)
        .as_bytes();
    let expected_key = CONFIG.api_key.as_bytes();

    if provided_key.len() != expected_key.len() || provided_key.ct_eq(expected_key).unwrap_u8() != 1
    {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(req).await)
}

async fn health_check() -> StatusCode {
    StatusCode::OK
}

async fn ready_check(db: Database) -> StatusCode {
    match sqlx::query("SELECT 1").execute(&db.0).await {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::SERVICE_UNAVAILABLE,
    }
}

pub async fn get_router() -> Router {
    let client = get_postgres_pool().await;
    let ready_client = client.clone();

    let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    let app_router = Router::new()
        .nest("/users/", users::get_router())
        .nest("/languages/", languages::get_router())
        .nest("/donate_notifications/", donate_notifications::get_router())
        .layer(middleware::from_fn(auth))
        .layer(Extension(client))
        .layer(prometheus_layer);

    let metric_router = Router::new()
        .route("/metrics", get(|| async move { metric_handle.render() }))
        .layer(middleware::from_fn(auth));

    let health_router = Router::new()
        .route("/health", get(health_check))
        .route("/ready", get(ready_check))
        .layer(Extension(ready_client));

    Router::new()
        .merge(app_router)
        .merge(metric_router)
        .merge(health_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(trace::DefaultMakeSpan::new().level(Level::INFO))
                .on_response(trace::DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(CatchPanicLayer::new())
}
