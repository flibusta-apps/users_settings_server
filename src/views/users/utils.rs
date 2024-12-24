use crate::views::Database;

pub async fn update_languages(user: i32, new_langs: Vec<String>, db: Database) {
    sqlx::query!(
        r#"
        DELETE FROM users_languages
        WHERE "user" = $1 AND language NOT IN (
            SELECT id FROM languages WHERE code = ANY($2)
        )
        "#,
        user,
        &new_langs
    )
    .execute(&db.0)
    .await
    .unwrap();

    sqlx::query!(
        r#"
        INSERT INTO users_languages ("user", language)
        SELECT $1, id
        FROM languages
        WHERE code = ANY($2)
        ON CONFLICT DO NOTHING
        "#,
        user,
        &new_langs
    )
    .execute(&db.0)
    .await
    .unwrap();
}
