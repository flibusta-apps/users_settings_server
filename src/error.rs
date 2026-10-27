use axum::{
    extract::{
        rejection::{JsonRejection, PathRejection, QueryRejection},
        FromRequest, FromRequestParts,
    },
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};

#[derive(Debug)]
pub enum AppError {
    Sqlx(sqlx::Error),
    Validation(String),
    Rejection(StatusCode, String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Sqlx(err) => {
                // Reported to Sentry via the tracing layer (ERROR events); do not capture twice.
                tracing::error!("DB error: {:?}", err);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({"detail": "Internal Server Error"})),
                )
                    .into_response()
            }
            AppError::Validation(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                Json(serde_json::json!({"detail": msg})),
            )
                .into_response(),
            AppError::Rejection(status, msg) => {
                (status, Json(serde_json::json!({"detail": msg}))).into_response()
            }
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Sqlx(err)
    }
}

impl From<JsonRejection> for AppError {
    fn from(r: JsonRejection) -> Self {
        AppError::Rejection(r.status(), r.body_text())
    }
}

impl From<QueryRejection> for AppError {
    fn from(r: QueryRejection) -> Self {
        AppError::Rejection(r.status(), r.body_text())
    }
}

impl From<PathRejection> for AppError {
    fn from(r: PathRejection) -> Self {
        AppError::Rejection(r.status(), r.body_text())
    }
}

/// `Json` extractor whose rejections are rendered as `{"detail": ...}`.
#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct ApiJson<T>(pub T);

/// `Query` extractor whose rejections are rendered as `{"detail": ...}`.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Query), rejection(AppError))]
pub struct ApiQuery<T>(pub T);

/// `Path` extractor whose rejections are rendered as `{"detail": ...}`.
#[derive(FromRequestParts)]
#[from_request(via(axum::extract::Path), rejection(AppError))]
pub struct ApiPath<T>(pub T);
