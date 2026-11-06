use serde_json::json;
use sqlx::PgPool;
use users_settings_server::views::users::{patch_user, serializers::PatchUserData};

fn patch(value: serde_json::Value) -> PatchUserData {
    serde_json::from_value(value).unwrap()
}

async fn seed(pool: &PgPool) {
    sqlx::query("INSERT INTO languages (label, code) VALUES ('Russian', 'ru'), ('English', 'en') ON CONFLICT (code) DO NOTHING")
        .execute(pool).await.unwrap();
    sqlx::query("INSERT INTO user_settings (user_id, last_name, first_name, username, source, default_search) VALUES (5000000000, 'last', 'first', 'name', 'bot', 'book')")
        .execute(pool).await.unwrap();
    patch_user(pool, 5_000_000_000, patch(json!({"allowed_langs": ["ru"]})))
        .await
        .unwrap()
        .unwrap();
}

#[sqlx::test(migrations = "./migrations")]
async fn concurrent_disjoint_patches_preserve_all_changes(pool: PgPool) {
    seed(&pool).await;
    let (languages, filename, search) = tokio::join!(
        patch_user(
            &pool,
            5_000_000_000,
            patch(json!({"allowed_langs": ["en", "en"]}))
        ),
        patch_user(
            &pool,
            5_000_000_000,
            patch(json!({"file_name_lang": "original"}))
        ),
        patch_user(
            &pool,
            5_000_000_000,
            patch(json!({"default_search": "author"}))
        )
    );
    languages.unwrap().unwrap();
    filename.unwrap().unwrap();
    search.unwrap().unwrap();
    let user = patch_user(&pool, 5_000_000_000, patch(json!({})))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.file_name_lang, "original");
    assert_eq!(user.default_search.as_deref(), Some("author"));
    assert_eq!(user.allowed_langs.len(), 1);
    assert_eq!(user.allowed_langs[0].code, "en");
    assert_eq!(user.first_name, "first");
    assert_eq!(user.source, "bot");
}

#[sqlx::test(migrations = "./migrations")]
async fn null_clears_search_and_empty_languages_clear_selection(pool: PgPool) {
    seed(&pool).await;
    let user = patch_user(
        &pool,
        5_000_000_000,
        patch(json!({"default_search": null, "allowed_langs": []})),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(user.default_search.is_none());
    assert!(user.allowed_langs.is_empty());
    assert_eq!(user.file_name_lang, "normalized");
}

#[sqlx::test(migrations = "./migrations")]
async fn invalid_patches_roll_back_and_missing_users_are_not_created(pool: PgPool) {
    seed(&pool).await;
    for data in [
        json!({"default_search": "invalid"}),
        json!({"file_name_lang": "invalid"}),
        json!({"file_name_lang": "original", "allowed_langs": ["unknown"]}),
    ] {
        assert!(patch_user(&pool, 5_000_000_000, patch(data)).await.is_err());
    }
    let user = patch_user(&pool, 5_000_000_000, patch(json!({})))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(user.file_name_lang, "normalized");
    assert_eq!(user.default_search.as_deref(), Some("book"));
    assert_eq!(user.allowed_langs[0].code, "ru");
    assert!(
        patch_user(&pool, 42, patch(json!({"file_name_lang": "original"})))
            .await
            .unwrap()
            .is_none()
    );
}
