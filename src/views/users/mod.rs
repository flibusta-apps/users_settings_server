pub mod serializers;
pub mod utils;

use crate::prisma::{language_to_user, user_activity, user_settings};
use axum::{
    extract::{Path, Query},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};

use self::{
    serializers::{CreateOrUpdateUserData, UserDetail},
    utils::update_languages,
};

use super::{
    pagination::{Page, Pagination},
    Database,
};

async fn get_users(pagination: Query<Pagination>, db: Database) -> impl IntoResponse {
    let pagination: Pagination = pagination.0;

    let users_count = db.user_settings().count(vec![]).exec().await.unwrap();

    let users: Vec<UserDetail> = db
        .user_settings()
        .find_many(vec![])
        .with(user_settings::languages::fetch(vec![]).with(language_to_user::language::fetch()))
        .order_by(user_settings::id::order(prisma_client_rust::Direction::Asc))
        .skip(pagination.skip())
        .take(pagination.take())
        .exec()
        .await
        .unwrap()
        .into_iter()
        .map(|item| item.into())
        .collect();

    Json(Page::create(users, users_count, pagination)).into_response()
}

async fn get_user(Path(user_id): Path<i64>, db: Database) -> impl IntoResponse {
    let user = db
        .user_settings()
        .find_unique(user_settings::user_id::equals(user_id))
        .with(user_settings::languages::fetch(vec![]).with(language_to_user::language::fetch()))
        .exec()
        .await
        .unwrap();

    if user.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }

    Json::<UserDetail>(user.unwrap().into()).into_response()
}

async fn create_or_update_user(
    db: Database,
    Json(data): Json<CreateOrUpdateUserData>,
) -> impl IntoResponse {
    let user = db
        .user_settings()
        .upsert(
            user_settings::user_id::equals(data.user_id),
            user_settings::create(
                data.user_id,
                data.last_name.clone(),
                data.first_name.clone(),
                data.username.clone(),
                data.source.clone(),
                vec![],
            ),
            vec![
                user_settings::last_name::set(data.last_name),
                user_settings::first_name::set(data.first_name),
                user_settings::username::set(data.username),
                user_settings::source::set(data.source),
            ],
        )
        .with(user_settings::languages::fetch(vec![]).with(language_to_user::language::fetch()))
        .exec()
        .await
        .unwrap();

    let user_id = user.id;
    update_languages(user, data.allowed_langs, db.clone()).await;

    let user = db
        .user_settings()
        .find_unique(user_settings::id::equals(user_id))
        .with(user_settings::languages::fetch(vec![]).with(language_to_user::language::fetch()))
        .exec()
        .await
        .unwrap()
        .unwrap();

    Json::<UserDetail>(user.into()).into_response()
}

async fn update_activity(Path(user_id): Path<i64>, db: Database) -> impl IntoResponse {
    let user = db
        .user_settings()
        .find_unique(user_settings::user_id::equals(user_id))
        .exec()
        .await
        .unwrap();

    let user = match user {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let _ = db
        .user_activity()
        .upsert(
            user_activity::user_id::equals(user.id),
            user_activity::create(
                chrono::offset::Local::now().into(),
                user_settings::id::equals(user.id),
                vec![],
            ),
            vec![user_activity::updated::set(
                chrono::offset::Local::now().into(),
            )],
        )
        .exec()
        .await;

    StatusCode::OK.into_response()
}

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get_users))
        .route("/:user_id", get(get_user))
        .route("/", post(create_or_update_user))
        .route("/:user_id/update_activity", post(update_activity))
}
