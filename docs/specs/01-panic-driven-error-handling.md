# Spec 01: Panic-driven error handling crashes the whole process

- **Priority:** high
- **Effort:** M
- **Category:** reliability

## Problem(s)

### 01.1 Every handler `.unwrap()`s database results
All request handlers unwrap sqlx results instead of mapping errors to HTTP responses:
- `src/views/users/mod.rs:27,62,98,134,168,185,203` (e.g. `.fetch_all(&db.0).await.unwrap()`)
- `src/views/languages.rs:17,30`
- `src/views/donate_notifications.rs:38,67`
- `src/views/users/utils.rs:16,31`

Any transient DB error (pool timeout, connection reset, constraint violation) panics inside the handler.

**Fix:** Introduce an `AppError(anyhow::Error)` / `sqlx::Error` wrapper implementing `IntoResponse` (500 + Sentry event), change handlers to return `Result<impl IntoResponse, AppError>` and use `?`.

### 01.2 `panic = 'abort'` turns any handler panic into full process death
`Cargo.toml:13`: `panic = 'abort'` in `[profile.release]`. With abort-on-panic there is no unwinding, so a single failed `.unwrap()` in one request (see 01.1) kills the entire server for all users, and there is no `CatchPanicLayer` either way.

**Fix:** Remove `panic = 'abort'` (or keep it but eliminate all request-path unwraps per 01.1 first) and add `tower_http::catch_panic::CatchPanicLayer` as defense in depth.

### 01.3 Pagination `page=0` panics (remote crash vector)
`src/views/pagination.rs:19-21`:
```rust
pub fn skip(&self) -> i64 {
    ((self.page - 1) * self.size).try_into().unwrap()
}
```
`page` is `usize` with default 1 but no minimum. `GET /users/?page=0` computes `0 - 1` → underflow: panic in debug, wrap-around in release, where the huge value then fails `try_into::<i64>()` and panics on `.unwrap()`. Combined with 01.2 this is a one-request denial of service for any API-key holder. `size` is also unbounded (`?size=10000000` is accepted).

**Fix:** Validate `page >= 1` and `1 <= size <= MAX_PAGE_SIZE` (e.g. 500), return 422 on violation; compute skip with `saturating_sub`.

### 01.4 No input validation; DB CHECK violations surface as panics
`src/views/users/serializers.rs:37-46` (`CreateOrUpdateUserData`) accepts arbitrary strings. Columns are `VARCHAR(64)/(32)` (`migrations/20240101000001_create_user_settings.sql:5-8`) and `default_search`/`file_name_lang` have CHECK constraints (`migrations/20240101000006`, `20240101000007`). Overlong or invalid values (e.g. `"default_search": "foo"`) make the INSERT at `src/views/users/mod.rs:115-134` fail and panic instead of returning 422.

**Fix:** Validate lengths and enum values (`book|author|series|translator`, `normalized|original`) in the handler and return 422 with a message; keep the CHECKs as backstop.

### 01.5 Config parsing panics with unhelpful messages
`src/config.rs:36`: `get_env("POSTGRES_PORT").parse().unwrap()` — a malformed port panics without naming the variable. Startup-time panics are acceptable, but the message should identify the cause.

**Fix:** `parse().unwrap_or_else(|_| panic!("Invalid POSTGRES_PORT"))`.

## Acceptance criteria
- No `.unwrap()`/`.expect()` on fallible operations in request handlers; DB errors return 500 JSON without process termination.
- `GET /users/?page=0`, `?page=1&size=0`, and `?size=100000` return 4xx, never a panic; server stays alive.
- POST `/users/` with `default_search: "invalid"` or a 300-char `first_name` returns 422.
- A killed DB connection during a request results in a 500 response and the server continues serving `/health`.
