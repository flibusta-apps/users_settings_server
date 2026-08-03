use sqlx::{Postgres, Transaction};

pub async fn update_languages(
    tx: &mut Transaction<'_, Postgres>,
    user: i32,
    new_langs: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query!(
        r#"
        DELETE FROM users_languages
        WHERE "user" = $1 AND language NOT IN (
            SELECT id FROM languages WHERE code = ANY($2)
        )
        "#,
        user,
        new_langs
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO users_languages ("user", language)
        SELECT $1, id
        FROM languages
        WHERE code = ANY($2)
        ON CONFLICT ("user", language) DO NOTHING
        "#,
        user,
        new_langs
    )
    .execute(&mut **tx)
    .await?;

    Ok(())
}
