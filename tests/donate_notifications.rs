//! These tests use `#[sqlx::test]`, which creates an isolated temporary database
//! per test (derived from the `DATABASE_URL` superuser connection, as in CI) and
//! applies the migrations. They never touch the database in `DATABASE_URL`
//! itself, so run them only against a disposable Postgres instance.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use users_settings_server::views::donate_notifications::{is_need_send_check, try_claim};

#[sqlx::test(migrations = "./migrations")]
async fn test_is_need_send_check_unknown_chat_returns_true(pool: PgPool) {
    let chat_id = -900_000_001i64;
    let result = is_need_send_check(&pool, chat_id, false).await.unwrap();
    assert!(result);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_is_need_send_check_group_within_window_returns_false(pool: PgPool) {
    let chat_id = -900_000_002i64;
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
}

#[sqlx::test(migrations = "./migrations")]
async fn test_is_need_send_check_group_outside_window_returns_true(pool: PgPool) {
    let chat_id = -900_000_003i64;
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
}

#[sqlx::test(migrations = "./migrations")]
async fn test_is_need_send_check_private_within_window_returns_false(pool: PgPool) {
    let chat_id = -900_000_004i64;
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
}

#[sqlx::test(migrations = "./migrations")]
async fn test_is_need_send_check_private_outside_window_returns_true(pool: PgPool) {
    let chat_id = -900_000_005i64;
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
}

#[sqlx::test(migrations = "./migrations")]
async fn test_try_claim_unknown_chat_then_immediate_retry(pool: PgPool) {
    let chat_id = -900_000_006i64;
    let first = try_claim(&pool, chat_id, false).await.unwrap();
    assert!(first);

    let second = try_claim(&pool, chat_id, false).await.unwrap();
    assert!(!second);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_try_claim_concurrent_only_one_succeeds(pool: PgPool) {
    let chat_id = -900_000_007i64;
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
}
