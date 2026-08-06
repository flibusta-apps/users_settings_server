# Spec 02: Migrations do not create everything the code depends on

- **Priority:** high
- **Effort:** M
- **Category:** correctness

## Problem(s)

### 02.1 Composite type `user_language_type` is never created by migrations
Queries cast to a Postgres composite type:
- `src/views/users/mod.rs:46` — `)::user_language_type),` (also lines 85, 155)
- `src/views/users/serializers.rs:3-5` — `#[sqlx(type_name = "user_language_type")]`

`grep -i "CREATE TYPE" migrations/` returns nothing — none of the 7 migrations creates the type. The project compiles thanks to the committed `.sqlx/` offline cache, but on a fresh database (migrations run at startup, `src/db.rs:35-38`) every `/users/` query fails at runtime with `type "user_language_type" does not exist`. The schema only works on the legacy production DB where the type was created manually.

**Fix:** Add a migration `CREATE TYPE user_language_type AS (id INTEGER, label VARCHAR(16), code VARCHAR(4));` guarded with a `DO $$ ... IF NOT EXISTS` block (matching the idempotent style of the other migrations). Verify a fresh `docker run postgres` + service start serves `/users/`.

### 02.2 `users_languages` lacks a UNIQUE constraint on ("user", language)
`migrations/20240101000004_create_users_languages.sql:1-6` creates only `id SERIAL PRIMARY KEY, language INTEGER NOT NULL, "user" INTEGER NOT NULL` — no unique pair constraint, and no index on `"user"` for the JOIN in `src/views/users/mod.rs:50`. This makes the `ON CONFLICT DO NOTHING` in `src/views/users/utils.rs:24` a no-op (nothing can conflict) — see Spec 03 for the resulting duplicate-row bug.

**Fix:** Migration that dedupes existing rows, then `ALTER TABLE users_languages ADD CONSTRAINT users_languages_user_language_key UNIQUE ("user", language);` (the unique index also serves the join).

### 02.3 Redundant duplicate indexes
`migrations/20240101000001_create_user_settings.sql:12`, `...0002...:9`, `...0003...:9`, `...0005...:9` each create a `UNIQUE INDEX ... IF NOT EXISTS x_key` on a column already declared `UNIQUE` in the CREATE TABLE above (the constraint already creates an index with that exact name). Harmless but misleading dead SQL.

**Fix:** Drop the redundant `CREATE UNIQUE INDEX` statements (safe since names collide with the constraint-created indexes anyway).

### 02.4 Naive timestamps (`TIMESTAMP WITHOUT TIME ZONE`)
`migrations/20240101000003_create_user_activity.sql:5` and `migrations/20240101000005_create_chat_donate_notifications.sql:5` use `TIMESTAMP WITHOUT TIME ZONE`, populated from mixed sources: `NOW()` (DB server time, `src/views/users/mod.rs:195`) and `chrono::offset::Local::now().naive_local()` (app container time, `src/views/donate_notifications.rs:63`). If DB and app containers have different timezones the values are incomparable (see Spec 05).

**Fix:** Migrate both columns to `TIMESTAMPTZ`; write with `NOW()` server-side or `chrono::Utc::now()`.

## Acceptance criteria
- Starting the service against an empty Postgres database succeeds and `GET /users/`, `POST /users/`, `GET /users/{id}` all work.
- `\d users_languages` shows a UNIQUE constraint on ("user", language).
- All timestamp columns are `timestamptz`; existing data migrated with an explicit assumed zone.
- Migrations remain idempotent when re-run against the current production schema.
