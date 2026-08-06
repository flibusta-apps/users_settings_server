# Spec 03: User languages sync — duplicates, no transaction, NULL-row decode panic

- **Priority:** high
- **Effort:** M
- **Category:** correctness

## Problem(s)

### 03.1 `update_languages` accumulates duplicate rows on every settings update
`src/views/users/utils.rs:18-31`:
```sql
INSERT INTO users_languages ("user", language)
SELECT $1, id FROM languages WHERE code = ANY($2)
ON CONFLICT DO NOTHING
```
The DELETE above it (utils.rs:4-16) only removes languages *not* in the new set; the INSERT then re-inserts *all* languages in the new set, including ones the user already has. Since `users_languages` has no unique constraint on ("user", language) (`migrations/20240101000004`), `ON CONFLICT DO NOTHING` never fires and a duplicate row is inserted for every already-present language on every `POST /users/`. `allowed_langs` in responses grows unboundedly with repeats (book_bot posts settings on every settings change).

**Fix:** Add the UNIQUE constraint (Spec 02.2) so `ON CONFLICT DO NOTHING` works, plus a data-fix migration deduplicating existing rows; or exclude already-present pairs in the INSERT (`AND NOT EXISTS (...)`).

### 03.2 Upsert + language sync run outside a transaction
`src/views/users/mod.rs:115-136`: the `user_settings` upsert, then `update_languages` (itself two separate statements executed directly on the pool, `utils.rs:14,30`), then the re-select at mod.rs:138-168 are four independent statements. A failure midway (or a concurrent POST for the same user) leaves settings and languages inconsistent, and the response may not reflect what was written.

**Fix:** Wrap upsert + DELETE + INSERT + final SELECT in a single `pool.begin()` transaction; pass `&mut *tx` into `update_languages`.

### 03.3 `ARRAY_AGG` without `FILTER` produces an undecodable NULL row for users with no languages
`src/views/users/mod.rs:41-48` (`get_users`) and `mod.rs:150-157` (re-select in `create_or_update_user`):
```sql
COALESCE(ARRAY_AGG(ROW(languages.id, languages.label, languages.code)::user_language_type),
         ARRAY[]::user_language_type[])
```
Unlike `get_user` (mod.rs:85), which correctly adds `FILTER (WHERE languages.id IS NOT NULL)`, these two queries aggregate the LEFT-JOIN NULL row into `ROW(NULL,NULL,NULL)::user_language_type`. The aggregate is non-NULL, so COALESCE does not help, and decoding into `UserLanguage { id: i32, label: String, code: String }` (`serializers.rs:5-9`) fails → `.unwrap()` panic (mod.rs:62/168) → process abort (Spec 01.2). Trigger: any user with zero `users_languages` rows appears on `GET /users/`, or `POST /users/` with `"allowed_langs": []`.

**Fix:** Add `FILTER (WHERE languages.id IS NOT NULL)` to both queries, mirroring `get_user`; regenerate the `.sqlx` offline cache.

## Acceptance criteria
- Posting the same settings twice leaves the `users_languages` row count for that user unchanged; `allowed_langs` in the response contains no duplicates.
- `POST /users/` with `"allowed_langs": []` returns 200 with `"allowed_langs": []`; `GET /users/` succeeds when such a user exists.
- Killing the DB connection mid-update leaves no partial state (settings updated but languages not).
- Regression test covering repeated updates and the empty-langs case.
