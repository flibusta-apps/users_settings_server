# Spec 09: Hot-path performance — missing JOIN index, pool defaults, request fan-out

- **Priority:** high
- **Effort:** S-M
- **Category:** performance

Context: book_bot calls this service on effectively every user update — `POST /users/` and `POST /users/{id}/update_activity` are the hottest endpoints in the whole system. Small per-request costs multiply accordingly.

## Problem(s)

### 09.1 No index on `users_languages."user"` — every user read walks the table

`migrations/20240101000004_create_users_languages.sql`: the table has only `id SERIAL PRIMARY KEY` and two foreign keys. Postgres does **not** create indexes for FK columns. Every `LEFT JOIN users_languages ON user_settings.id = users_languages."user"` (`src/views/users/mod.rs:50,160` and the delete in `utils.rs`) therefore scans `users_languages` — per request, on the hottest queries in the service, with cost growing linearly with the user base. The FK's `ON DELETE CASCADE` from `user_settings` also scans on every user deletion.

**Fix:** New migration: `CREATE INDEX IF NOT EXISTS ix_users_languages_user ON users_languages ("user");` — or, better, the composite `UNIQUE ("user", language)` already proposed by Spec 02.2, which covers this lookup *and* fixes the duplicate-rows bug; implement them together.

### 09.2 Pool defaults: 300-second acquire timeout

`src/config.rs:43-46`: `POSTGRES_POOL_ACQUIRE_TIMEOUT_SEC` defaults to **300**. Under pool exhaustion (10 connections, default), each handler waits up to 5 minutes while book_bot has long since timed out — the classic amplification cascade (same defect class as telegram_files_cache_server Spec 04.5).

**Fix:** Change the default to 5 s. Add `.min_connections(2)` so a cold start / idle-reaped pool doesn't pay connection setup on the first burst.

### 09.3 `POST /users/` costs four sequential DB round trips

`src/views/users/mod.rs:108-170` + `src/views/users/utils.rs:4-31`: UPSERT into `user_settings` → DELETE stale languages → INSERT languages → full re-SELECT with JOIN/ARRAY_AGG for the response body. Four awaited round trips per call (each also a pool acquire), non-transactional (races covered by Spec 03.2).

**Fix:** When wrapping in a transaction for Spec 03.2, also collapse the work: run the three writes on one transaction/connection, and build the response from data already in hand (the UPSERT's `RETURNING` row plus the language list from the request — after a successful commit they *are* the current state), dropping the final SELECT. Result: one connection, one round trip fewer, and no read-after-write inconsistency window.

### 09.4 `GET /languages/` hits the DB on every call

`src/views/languages.rs:14-15`: the full language list (a few dozen rows, changes ~never) is fetched per request. book_bot requests it constantly for settings keyboards.

**Fix:** Process-level cache: `moka` (or a `tokio::sync::RwLock<Arc<Vec<LanguageDetail>>>`) with a 5–15 min TTL, invalidated on any language mutation endpoint if one exists. Alternatively set `Cache-Control: max-age=...` and cache in book_bot — decide once, document in the API contract.

### 09.5 Pagination runs a separate unfiltered `COUNT(*)` per page (minor)

`src/views/users/mod.rs:24-26,54-55`: each list page issues `COUNT(*)` over `user_settings` plus the page SELECT. Low traffic today; fold `COUNT(*) OVER ()` into the page query if this endpoint ever gets hot. `src/views/pagination.rs:50` also computes pages via f64 — use integer ceil-div `(count + size - 1) / size`.

### 09.6 Hygiene: tokio full features, per-call string defaults

`Cargo.toml`: `tokio = { features = ["full"] }` → enumerate used features. `src/views/users/mod.rs:113`: `unwrap_or_else(|| "normalized".to_string())` allocates on the hot path — compare as `&str` / use `Cow`. Bundle with other work.

## Acceptance criteria

- `EXPLAIN` on the user-detail query shows an index scan on `users_languages` (no Seq Scan); migration applied together with or after Spec 02.2's unique constraint.
- Default acquire timeout ≤ 5 s; a saturated pool returns errors within that budget (test with pool size 1 and two concurrent slow queries).
- `POST /users/` executes on a single transaction with at most 3 statements and no trailing re-SELECT; response body unchanged (snapshot test against the current shape).
- `GET /languages/` served from cache: two consecutive calls produce one DB query (verified via query counter or logs).
