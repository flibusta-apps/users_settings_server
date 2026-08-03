use sqlx::PgPool;
use users_settings_server::views::users::utils::update_languages;

#[sqlx::test(migrations = "./migrations")]
async fn test_repeated_update_with_duplicates_does_not_duplicate_rows(pool: PgPool) {
    // Seed languages.
    sqlx::query!(
        r#"INSERT INTO languages (label, code) VALUES ('Ukrainian', 'uk'), ('Belarusian', 'be'), ('Russian', 'ru')"#
    )
    .execute(&pool)
    .await
    .unwrap();

    // Seed a user_settings row.
    let user = sqlx::query!(
        r#"
        INSERT INTO user_settings (user_id, last_name, first_name, username, source)
        VALUES ($1, 'last', 'first', 'username', 'source')
        RETURNING id
        "#,
        1i64,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let new_langs = vec!["uk".to_string(), "be".to_string(), "uk".to_string()];

    // First "POST".
    let mut tx = pool.begin().await.unwrap();
    update_languages(&mut tx, user.id, &new_langs)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    // Second "POST" with the same duplicate-containing list.
    let mut tx = pool.begin().await.unwrap();
    update_languages(&mut tx, user.id, &new_langs)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM users_languages WHERE "user" = $1"#,
        user.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(count, 2, "expected exactly 2 rows (no duplicates)");

    let mut codes = sqlx::query_scalar!(
        r#"
        SELECT languages.code
        FROM users_languages
        JOIN languages ON users_languages.language = languages.id
        WHERE users_languages."user" = $1
        "#,
        user.id,
    )
    .fetch_all(&pool)
    .await
    .unwrap();
    codes.sort();

    assert_eq!(codes, vec!["be".to_string(), "uk".to_string()]);
}

#[sqlx::test(migrations = "./migrations")]
async fn test_empty_allowed_langs_results_in_no_rows(pool: PgPool) {
    sqlx::query!(r#"INSERT INTO languages (label, code) VALUES ('Ukrainian', 'uk')"#)
        .execute(&pool)
        .await
        .unwrap();

    let user = sqlx::query!(
        r#"
        INSERT INTO user_settings (user_id, last_name, first_name, username, source)
        VALUES ($1, 'last', 'first', 'username', 'source')
        RETURNING id
        "#,
        2i64,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    let empty_langs: Vec<String> = vec![];

    let mut tx = pool.begin().await.unwrap();
    update_languages(&mut tx, user.id, &empty_langs)
        .await
        .unwrap();
    tx.commit().await.unwrap();

    let count = sqlx::query_scalar!(
        r#"SELECT COUNT(*) AS "count!" FROM users_languages WHERE "user" = $1"#,
        user.id,
    )
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(count, 0);
}
