//! These tests connect directly to the DATABASE_URL configured in `.env` and
//! operate on dedicated, out-of-range `chat_id` values inside the real
//! `chat_donate_notifications` table (cleaned up before/after each test).
//!
//! Note: unlike `tests/user_languages_dedup.rs`, these tests intentionally do
//! NOT use the `#[sqlx::test]` macro. That macro requires the connecting role
//! to have `CREATEDB` privilege (to spin up an ephemeral per-test database),
//! which the `flibusta_users_settings_user` role configured in `.env` does not
//! have. Connecting directly to the existing database and scoping test data
//! by a dedicated chat_id range keeps these tests runnable in this
//! environment while still exercising the real Postgres instance.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use users_settings_server::views::donate_notifications::{is_need_send_check, try_claim};

async fn connect_pool() -> PgPool {
    let _ = dotenvy::dotenv();
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    PgPool::connect(&url)
        .await
        .expect("failed to connect to database")
}

async fn cleanup(pool: &PgPool, chat_id: i64) {
    let _ = sqlx::query!(
        r#"DELETE FROM chat_donate_notifications WHERE chat_id = $1"#,
        chat_id
    )
    .execute(pool)
    .await;
}

#[tokio::test]
async fn test_is_need_send_check_unknown_chat_returns_true() {
    let pool = connect_pool().await;
    let chat_id = -900_000_001i64;
    cleanup(&pool, chat_id).await;

    let result = is_need_send_check(&pool, chat_id, false).await.unwrap();
    assert!(result);

    cleanup(&pool, chat_id).await;
}

#[tokio::test]
async fn test_is_need_send_check_group_within_window_returns_false() {
    let pool = connect_pool().await;
    let chat_id = -900_000_002i64;
    cleanup(&pool, chat_id).await;

    sqlx::query!(
        r#"INSERT INTO chat_donate_notifications (chat_id, sended) VALUES ($1, $2)"#,
        chat_id,
        Utc::now(),
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_need_send_check(&pool, chat_id, false).await.unwrap();
    assert!(!result);

    cleanup(&pool, chat_id).await;
}

#[tokio::test]
async fn test_is_need_send_check_group_outside_window_returns_true() {
    let pool = connect_pool().await;
    let chat_id = -900_000_003i64;
    cleanup(&pool, chat_id).await;

    sqlx::query!(
        r#"INSERT INTO chat_donate_notifications (chat_id, sended) VALUES ($1, $2)"#,
        chat_id,
        Utc::now() - Duration::days(8),
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_need_send_check(&pool, chat_id, false).await.unwrap();
    assert!(result);

    cleanup(&pool, chat_id).await;
}

#[tokio::test]
async fn test_is_need_send_check_private_within_window_returns_false() {
    let pool = connect_pool().await;
    let chat_id = -900_000_004i64;
    cleanup(&pool, chat_id).await;

    sqlx::query!(
        r#"INSERT INTO chat_donate_notifications (chat_id, sended) VALUES ($1, $2)"#,
        chat_id,
        Utc::now() - Duration::days(30),
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_need_send_check(&pool, chat_id, true).await.unwrap();
    assert!(!result);

    cleanup(&pool, chat_id).await;
}

#[tokio::test]
async fn test_is_need_send_check_private_outside_window_returns_true() {
    let pool = connect_pool().await;
    let chat_id = -900_000_005i64;
    cleanup(&pool, chat_id).await;

    sqlx::query!(
        r#"INSERT INTO chat_donate_notifications (chat_id, sended) VALUES ($1, $2)"#,
        chat_id,
        Utc::now() - Duration::days(61),
    )
    .execute(&pool)
    .await
    .unwrap();

    let result = is_need_send_check(&pool, chat_id, true).await.unwrap();
    assert!(result);

    cleanup(&pool, chat_id).await;
}

#[tokio::test]
async fn test_try_claim_unknown_chat_then_immediate_retry() {
    let pool = connect_pool().await;
    let chat_id = -900_000_006i64;
    cleanup(&pool, chat_id).await;

    let first = try_claim(&pool, chat_id, false).await.unwrap();
    assert!(first);

    let second = try_claim(&pool, chat_id, false).await.unwrap();
    assert!(!second);

    cleanup(&pool, chat_id).await;
}

#[tokio::test]
async fn test_try_claim_concurrent_only_one_succeeds() {
    let pool = connect_pool().await;
    let chat_id = -900_000_007i64;
    cleanup(&pool, chat_id).await;

    let pool_a = pool.clone();
    let pool_b = pool.clone();

    let handle_a = tokio::spawn(async move { try_claim(&pool_a, chat_id, false).await.unwrap() });
    let handle_b = tokio::spawn(async move { try_claim(&pool_b, chat_id, false).await.unwrap() });

    let (result_a, result_b) = tokio::try_join!(handle_a, handle_b).unwrap();

    assert_ne!(
        result_a, result_b,
        "expected exactly one true and one false, got a={result_a} b={result_b}"
    );
    assert!(
        result_a || result_b,
        "expected at least one claim to succeed"
    );

    cleanup(&pool, chat_id).await;
}
