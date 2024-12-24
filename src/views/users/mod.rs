pub mod serializers;
pub mod utils;

use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use serializers::SimpleUser;

use self::{
    serializers::{CreateOrUpdateUserData, UserDetail, UserLanguage},
    utils::update_languages,
};

use super::{
    pagination::{Page, Pagination},
    Database,
};

async fn get_users(pagination: Query<Pagination>, db: Database) -> impl IntoResponse {
    let pagination: Pagination = pagination.0;

    let users_count = sqlx::query_scalar(r#"SELECT COUNT(*) FROM user_settings"#)
        .fetch_one(&db.0)
        .await
        .unwrap();

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
            ARRAY_AGG((
                languages.id,
                languages.label,
                languages.code
            )) AS "allowed_langs: Vec<UserLanguage>"
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
    .await
    .unwrap();

    Json(Page::create(users, users_count, pagination)).into_response()
}

async fn get_user(Path(user_id): Path<i64>, db: Database) -> impl IntoResponse {
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
            ARRAY_AGG((
                languages.id,
                languages.label,
                languages.code
            )) AS "allowed_langs: Vec<UserLanguage>"
        FROM user_settings
        LEFT JOIN users_languages ON user_settings.id = users_languages.user
        LEFT JOIN languages ON users_languages.language = languages.id
        WHERE user_settings.user_id = $1
        GROUP BY user_settings.id
        "#,
        user_id,
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    if user.is_none() {
        return StatusCode::NO_CONTENT.into_response();
    }

    Json::<UserDetail>(user.unwrap()).into_response()
}

async fn create_or_update_user(
    db: Database,
    Json(data): Json<CreateOrUpdateUserData>,
) -> impl IntoResponse {
    let user = sqlx::query_as!(
        SimpleUser,
        r#"
            INSERT INTO user_settings (user_id, last_name, first_name, username, source)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (user_id) DO UPDATE
            SET last_name = $2, first_name = $3, username = $4, source = $5
            RETURNING id, user_id, last_name, first_name, username, source
        "#,
        data.user_id,
        data.last_name,
        data.first_name,
        data.username,
        data.source,
    )
    .fetch_one(&db.0)
    .await
    .unwrap();

    update_languages(user.id, data.allowed_langs, db.clone()).await;

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
            ARRAY_AGG((
                languages.id,
                languages.label,
                languages.code
            )) AS "allowed_langs: Vec<UserLanguage>"
        FROM user_settings
        LEFT JOIN users_languages ON user_settings.id = users_languages.user
        LEFT JOIN languages ON users_languages.language = languages.id
        WHERE user_settings.id = $1
        GROUP BY user_settings.id
        "#,
        user.id,
    )
    .fetch_one(&db.0)
    .await
    .unwrap();

    Json::<UserDetail>(user).into_response()
}

async fn update_activity(Path(user_id): Path<i64>, db: Database) -> impl IntoResponse {
    let user = sqlx::query_as!(
        SimpleUser,
        r#"
        SELECT id, user_id, last_name, first_name, username, source
        FROM user_settings
        WHERE user_id = $1
        "#,
        user_id,
    )
    .fetch_optional(&db.0)
    .await
    .unwrap();

    let user = match user {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
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
    .await
    .unwrap();

    StatusCode::OK.into_response()
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get_users))
        .route("/:user_id", get(get_user))
        .route("/", post(create_or_update_user))
        .route("/:user_id/update_activity", post(update_activity))
}
