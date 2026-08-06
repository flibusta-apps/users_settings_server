# Spec 05: Donate notifications — non-atomic check/mark and fragile time handling

- **Priority:** medium
- **Effort:** M
- **Category:** correctness

## Problem(s)

### 05.1 Check-then-mark is not atomic; duplicate notifications possible
The schedule itself is: notify again after 60 days for private chats, 7 days for groups (`src/views/donate_notifications.rs:28-29`), and an unknown chat is always "need send" (line 52) — that matches the intended contract. But the protocol is two separate calls: `GET /{chat_id}/is_need_send` (lines 23-54) and `POST /{chat_id}` mark-sent (lines 56-70). The consumer (book_bot, `book_bot/.../services/donation_notifications.rs:30-31`) does `if is_need_donate_notifications(...) { mark_donate_notification_sent(...) }` guarded only by an in-process moka cache (lines 18-28). Two bot replicas, or two concurrent updates racing the cache insert, both read `true` and both notify.

**Fix:** Add an atomic claim endpoint, e.g. `POST /donate_notifications/{chat_id}/try_claim?is_private=` executing a single statement:
`INSERT ... ON CONFLICT (chat_id) DO UPDATE SET sended = NOW() WHERE chat_donate_notifications.sended <= NOW() - $interval RETURNING ...` and returning `true` iff a row was written. Keep the old endpoints for compatibility until book_bot migrates.

### 05.2 Timestamps use app-container local time with naive storage
`src/views/donate_notifications.rs:48-49` compares against `chrono::offset::Local::now().naive_local()`; `mark_sent` writes `chrono::offset::Local::now().naive_local()` (line 63) into `sended TIMESTAMP WITHOUT TIME ZONE` (`migrations/20240101000005:5`). Correctness silently depends on every app container having the same TZ forever; a TZ change (or comparing against DB-written `NOW()` values elsewhere) shifts the schedule by hours. Since the deltas are 7/60 days the impact is small, but it is free to fix.

**Fix:** Use `chrono::Utc::now().naive_utc()` (or move the comparison into SQL with `NOW()`), and migrate the column to `TIMESTAMPTZ` (see Spec 02.4).

### 05.3 `mark_sent` does useless work and unwraps
`src/views/donate_notifications.rs:57-67` uses `query_as!(ChatDonateNotification, ... RETURNING sended)` + `fetch_one(...).unwrap()` but discards the result and returns plain `StatusCode::OK` (line 69). The `RETURNING`/`query_as!` is dead weight, and the `.unwrap()` is a crash path (Spec 01).

**Fix:** Use `sqlx::query!(...).execute()`, propagate errors, return 204 or 200 explicitly.

## Acceptance criteria
- A concurrency test (two parallel claim calls for the same chat_id) yields exactly one `true`.
- All donate-notification timestamps are written/compared in UTC; behavior is invariant to the container TZ.
- `is_need_send` returns `true` for unknown chats, `false` within the window (7d group / 60d private), `true` after — covered by tests.
- No `.unwrap()` in donate_notifications handlers.
