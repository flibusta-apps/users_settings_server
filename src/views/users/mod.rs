pub mod serializers;
pub mod utils;

use self::{
    serializers::{CreateOrUpdateUserData, SimpleUser, UserDetail, UserLanguage},
    utils::update_languages,
};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use super::{
    pagination::{Page, Pagination},
    Database,
};
use crate::error::AppError;

async fn get_users(
    pagination: Query<Pagination>,
    db: Database,
) -> Result<impl IntoResponse, AppError> {
    let pagination: Pagination = pagination.0;

    if let Err(msg) = pagination.validate() {
        return Err(AppError::Validation(msg));
    }

    let users_count = sqlx::query_scalar(r#"SELECT COUNT(*) FROM user_settings"#)
        .fetch_one(&db.0)
        .await?;

    let users = sqlx::query_as!(
        UserDetail,
        r#"
        SELECT
            user_settings.id,
            user_settings.user_id,
            user_settings.last_name,
            user_settings.first_name,
            user_settings.username,
            user_settings.source,
            user_settings.default_search,
            user_settings.file_name_lang,
            COALESCE(
                ARRAY_AGG(ROW(
                    languages.id,
                    languages.label,
                    languages.code
                )::user_language_type) FILTER (WHERE languages.id IS NOT NULL),
                ARRAY[]::user_language_type[]
            ) AS "allowed_langs!: Vec<UserLanguage>"
        FROM user_settings
        LEFT JOIN users_languages ON user_settings.id = users_languages.user
        LEFT JOIN languages ON users_languages.language = languages.id
        GROUP BY user_settings.id
        ORDER BY user_settings.id ASC
        OFFSET $1
        LIMIT $2
        "#,
        pagination.skip(),
        pagination.take(),
    )
    .fetch_all(&db.0)
    .await?;

    Ok(Json(Page::create(users, users_count, pagination)).into_response())
}

async fn get_user(Path(user_id): Path<i64>, db: Database) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        UserDetail,
        r#"
        SELECT
            user_settings.id,
            user_settings.user_id,
            user_settings.last_name,
            user_settings.first_name,
            user_settings.username,
            user_settings.source,
            user_settings.default_search,
            user_settings.file_name_lang,
            COALESCE(
                ARRAY_AGG(ROW(
                    languages.id,
                    languages.label,
                    languages.code
                )::user_language_type) FILTER (WHERE languages.id IS NOT NULL),
                ARRAY[]::user_language_type[]
            ) AS "allowed_langs!: Vec<UserLanguage>"
        FROM user_settings
        LEFT JOIN users_languages ON user_settings.id = users_languages.user
        LEFT JOIN languages ON users_languages.language = languages.id
        WHERE user_settings.user_id = $1
        GROUP BY user_settings.id
        "#,
        user_id,
    )
    .fetch_optional(&db.0)
    .await?;

    let user = match user {
        Some(v) => v,
        None => return Ok(StatusCode::NOT_FOUND.into_response()),
    };

    Ok(Json::<UserDetail>(user).into_response())
}

async fn create_or_update_user(
    db: Database,
    Json(data): Json<CreateOrUpdateUserData>,
) -> Result<impl IntoResponse, AppError> {
    if let Err(msg) = data.validate() {
        return Err(AppError::Validation(msg));
    }

    let file_name_lang = data
        .file_name_lang
        .clone()
        .unwrap_or_else(|| "normalized".to_string());

    let mut tx = db.0.begin().await?;

    let user = sqlx::query_as!(
        SimpleUser,
        r#"
            INSERT INTO user_settings (user_id, last_name, first_name, username, source, default_search, file_name_lang)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id) DO UPDATE
            SET last_name = $2, first_name = $3, username = $4, source = $5, default_search = $6, file_name_lang = $7
            RETURNING id, user_id, last_name, first_name, username, source, default_search, file_name_lang
        "#,
        data.user_id,
        data.last_name,
        data.first_name,
        data.username,
        data.source,
        data.default_search,
        file_name_lang,
    )
    .fetch_one(&mut *tx)
    .await?;

    update_languages(&mut tx, user.id, &data.allowed_langs).await?;

    let user = sqlx::query_as!(
        UserDetail,
        r#"
        SELECT
            user_settings.id,
            user_settings.user_id,
            user_settings.last_name,
            user_settings.first_name,
            user_settings.username,
            user_settings.source,
            user_settings.default_search,
            user_settings.file_name_lang,
            COALESCE(
                ARRAY_AGG(ROW(
                    languages.id,
                    languages.label,
                    languages.code
                )::user_language_type) FILTER (WHERE languages.id IS NOT NULL),
                ARRAY[]::user_language_type[]
            ) AS "allowed_langs!: Vec<UserLanguage>"
        FROM user_settings
        LEFT JOIN users_languages ON user_settings.id = users_languages.user
        LEFT JOIN languages ON users_languages.language = languages.id
        WHERE user_settings.id = $1
        GROUP BY user_settings.id
        "#,
        user.id,
    )
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json::<UserDetail>(user).into_response())
}

async fn update_activity(
    Path(user_id): Path<i64>,
    db: Database,
) -> Result<impl IntoResponse, AppError> {
    let user = sqlx::query_as!(
        SimpleUser,
        r#"
        SELECT id, user_id, last_name, first_name, username, source, default_search, file_name_lang
        FROM user_settings
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(&db.0)
    .await?;

    let user = match user {
        Some(v) => v,
        None => return Ok(StatusCode::NOT_FOUND.into_response()),
    };

    sqlx::query!(
        r#"
            INSERT INTO user_activity ("user", updated)
            VALUES ($1, NOW())
            ON CONFLICT ("user") DO UPDATE
            SET updated = NOW()
        "#,
        user.id,
    )
    .execute(&db.0)
    .await?;

    Ok(StatusCode::OK.into_response())
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get_users))
        .route("/{user_id}", get(get_user))
        .route("/", post(create_or_update_user))
        .route("/{user_id}/update_activity", post(update_activity))
}
