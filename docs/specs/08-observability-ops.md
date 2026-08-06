# Spec 08: Observability and operational hardening

- **Priority:** low
- **Effort:** S
- **Category:** observability

## Problem(s)

### 08.1 `/health` never checks the database
`src/views/mod.rs:42-44`:
```rust
async fn health_check() -> StatusCode { StatusCode::OK }
```
The service is useless without Postgres, yet health stays green when the pool is dead — orchestrators will keep routing traffic to a broken instance.

**Fix:** Keep `/health` as liveness; add `/ready` executing `SELECT 1` against the pool (503 on failure) and use it as the readiness/HEALTHCHECK probe.

### 08.2 Sentry is mandatory and misconfiguration panics at boot
`src/config.rs:49`: `sentry_dsn: get_env("SENTRY_DSN")` and `src/main.rs:26`: `Dsn::from_str(&config::CONFIG.sentry_dsn).unwrap()` — the service cannot start without a valid DSN, which hurts local dev and test environments. Also, since errors are panics (Spec 01) and the release profile aborts, Sentry rarely gets a useful event anyway.

**Fix:** Make `SENTRY_DSN` optional (empty → skip `sentry::init`), and report handler errors as `tracing::error!` events (picked up by the sentry layer at `src/main.rs:34-37`) once Spec 01 lands.

### 08.3 Pool acquire timeout defaults to 300 s
`src/config.rs:43-46`: `postgres_pool_acquire_timeout_sec` defaults to `300`. With 10 max connections, a stuck DB makes requests hang for five minutes each, piling up clients (book_bot's HTTP timeout is 30 s — `book_bot/.../user_settings/mod.rs:14` — so every caller gives up long before the pool does).

**Fix:** Default to <= 10 s; add a `min_connections`/`test_before_acquire` if reconnect storms are a concern.

### 08.4 `user_activity` is write-only dead weight via the API
`POST /users/{id}/update_activity` writes `user_activity` (`src/views/users/mod.rs:192-203`), but no endpoint ever reads the table. If it exists purely for ad-hoc analytics, document that; otherwise it is dead surface.

**Fix:** Document the consumer (SQL analytics) in README, or expose `last_activity` in `UserDetail`, or drop the endpoint+table.

### 08.5 Committed editor config
`.vscode/launch.json` is committed while `.gitignore:3` lists `.vscode` — inconsistent; harmless but noisy.

**Fix:** `git rm --cached .vscode/launch.json` or intentionally un-ignore it.

## Acceptance criteria
- `/ready` returns 503 when Postgres is stopped and 200 when it is back; `/health` stays 200 (process alive).
- Service starts locally without `SENTRY_DSN`.
- Default acquire timeout <= 10 s (still env-overridable).
- `user_activity` usage documented or surfaced through the API.
