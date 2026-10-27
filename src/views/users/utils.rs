use sqlx::{Postgres, Transaction};

use super::serializers::UserLanguage;
use crate::error::AppError;

/// Replaces the user's languages with `new_langs`.
///
/// All requested codes are validated against the `languages` table inside the
/// transaction; if any is unknown nothing is modified and a validation error
/// listing the unknown codes is returned. On success returns the languages that
/// were actually saved (in request order, without duplicates).
pub async fn update_languages(
    tx: &mut Transaction<'_, Postgres>,
    user: i32,
    new_langs: &[String],
) -> Result<Vec<UserLanguage>, AppError> {
    let mut seen = std::collections::HashSet::new();
    let requested: Vec<String> = new_langs
        .iter()
        .filter(|code| seen.insert(code.as_str()))
        .cloned()
        .collect();

    let found = sqlx::query_as!(
        UserLanguage,
        "SELECT id, label, code FROM languages WHERE code = ANY($1)",
        &requested
    )
    .fetch_all(&mut **tx)
    .await?;

    let unknown: Vec<&str> = requested
        .iter()
        .filter(|code| !found.iter().any(|l| &l.code == *code))
        .map(|code| code.as_str())
        .collect();
    if !unknown.is_empty() {
        return Err(AppError::Validation(format!(
            "unknown language codes: {}",
            unknown.join(", ")
        )));
    }

    let ids: Vec<i32> = found.iter().map(|l| l.id).collect();

    sqlx::query!(
        r#"DELETE FROM users_languages WHERE "user" = $1 AND language <> ALL($2)"#,
        user,
        &ids
    )
    .execute(&mut **tx)
    .await?;

    sqlx::query!(
        r#"
        INSERT INTO users_languages ("user", language)
        SELECT $1, UNNEST($2::int4[])
        ON CONFLICT ("user", language) DO NOTHING
        "#,
        user,
        &ids
    )
    .execute(&mut **tx)
    .await?;

    let mut saved = found;
    saved.sort_by_key(|l| requested.iter().position(|c| c == &l.code));
    Ok(saved)
}
