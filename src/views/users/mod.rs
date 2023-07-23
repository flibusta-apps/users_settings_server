pub mod serializers;
pub mod utils;

use axum::{Router, response::IntoResponse, routing::{get, post}, extract::{Query, Path, self}, Json, http::StatusCode};
use crate::{prisma::{user_settings, language_to_user, user_activity}, db::get_prisma_client};

use self::{serializers::{UserDetail, CreateOrUpdateUserData}, utils::update_languages};

use super::pagination::{Pagination, Page};


async fn get_users(
    pagination: Query<Pagination>
) -> impl IntoResponse {
    let pagination: Pagination = pagination.0;

    let client = get_prisma_client().await;

    let users_count = client.user_settings()
        .count(vec![])
        .exec()
        .await
        .unwrap();

    let users: Vec<UserDetail> = client.user_settings()
        .find_many(vec![])
        .with(
            user_settings::languages::fetch(vec![])
                .with(
                    language_to_user::language::fetch()
                )
        )
        .order_by(user_settings::id::order(prisma_client_rust::Direction::Asc))
        .skip(pagination.skip())
        .take(pagination.take())
        .exec()
        .await
        .unwrap()
        .into_iter()
        .map(|item| item.into())
        .collect();

    Json(Page::create(
        users,
        users_count,
        pagination
    )).into_response()
}


async fn get_user(
    Path(user_id): Path<i64>
) -> impl IntoResponse {
    let client = get_prisma_client().await;

    let user = client.user_settings()
        .find_unique(user_settings::user_id::equals(user_id))
        .with(
            user_settings::languages::fetch(vec![])
                .with(
                    language_to_user::language::fetch()
                )
        )
        .exec()
        .await
        .unwrap();

    if user.is_none() {
        return StatusCode::NOT_FOUND.into_response();
    }

    Json::<UserDetail>(user.unwrap().into()).into_response()
}


async fn create_or_update_user(
    extract::Json(data): extract::Json<CreateOrUpdateUserData>
) -> impl IntoResponse {
    let client = get_prisma_client().await;

    let user = client.user_settings()
        .upsert(
            user_settings::user_id::equals(data.user_id),
            user_settings::create(
                data.user_id,
                data.last_name.clone(),
                data.first_name.clone(),
                data.username.clone(),
                data.source.clone(),
                vec![]
            ),
            vec![
                user_settings::last_name::set(data.last_name),
                user_settings::first_name::set(data.first_name),
                user_settings::username::set(data.username),
                user_settings::source::set(data.source)
            ]
        )
        .with(
            user_settings::languages::fetch(vec![])
                .with(
                    language_to_user::language::fetch()
                )
        )
        .exec()
        .await
        .unwrap();

    let user_id = user.id;
    update_languages(user, data.allowed_langs).await;

    let user = client.user_settings()
        .find_unique(user_settings::id::equals(user_id))
        .with(
            user_settings::languages::fetch(vec![])
                .with(
                    language_to_user::language::fetch()
                )
        )
        .exec()
        .await
        .unwrap()
        .unwrap();

    Json::<UserDetail>(user.into()).into_response()
}


async fn update_activity(
    Path(user_id): Path<i64>,
) -> impl IntoResponse {
    let client = get_prisma_client().await;

    let user = client.user_settings()
        .find_unique(user_settings::user_id::equals(user_id))
        .exec()
        .await
        .unwrap();

    let user = match user {
        Some(v) => v,
        None => return StatusCode::NOT_FOUND.into_response(),
    };

    let _ = client.user_activity()
        .upsert(
            user_activity::user_id::equals(user.id),
            user_activity::create(
                chrono::offset::Local::now().into(),
                user_settings::id::equals(user.id),
                vec![]
            ),
            vec![
                user_activity::updated::set(chrono::offset::Local::now().into())
            ]
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