# Spec 04: Inconsistent "user not found" status codes (204 vs 404)

- **Priority:** medium
- **Effort:** S
- **Category:** correctness

## Problem(s)

### 04.1 `GET /users/{user_id}` returns 204, `POST /users/{user_id}/update_activity` returns 404
For a nonexistent user:
- `src/views/users/mod.rs:100-102`:
```rust
if user.is_none() {
    return StatusCode::NO_CONTENT.into_response();
}
```
- `src/views/users/mod.rs:187-190`:
```rust
None => return StatusCode::NOT_FOUND.into_response(),
```
So the answer to the "204 or 404?" confusion is: **both, depending on the endpoint**. The consumer (book_bot) handles them differently:
- `book_bot/book_bot/src/bots/approved_bot/services/user_settings/mod.rs:115-119` calls `.error_for_status()?` *then* checks `StatusCode::NO_CONTENT` — so 204 works for GET, and a 404 there would be an error (it never happens today).
- `book_bot/.../user_settings/mod.rs:206-217` (`update_user_activity`) calls `.error_for_status()?` with no 404 handling — every activity ping for a user who has not yet been created surfaces as an `anyhow` error in the bot instead of a benign "no such user".

Using 204 as "not found" is also semantically wrong (204 = success with no body) and breaks generic clients/monitoring that treat 2xx as success.

**Fix:** Standardize on 404 for missing resources on both endpoints. Coordinate a two-step rollout: (1) teach book_bot's `get_user_settings` to treat 404 as `Ok(None)` (keep the 204 branch during transition) and `update_user_activity` to ignore 404; (2) switch the server to 404. Document the contract in the repo (e.g. this spec / README route table).

### 04.2 `GET /languages/{code}` already uses 404 — keep as reference behavior
`src/views/languages.rs:34`: `None => StatusCode::NOT_FOUND.into_response()`. This is the convention the users endpoints should follow.

**Fix:** None (reference only); ensure Spec 04.1 aligns with it.

## Acceptance criteria
- `GET /users/{unknown}` and `POST /users/{unknown}/update_activity` return the same, documented status code (target: 404 with empty or JSON error body).
- book_bot `get_user_settings` returns `Ok(None)` and `update_user_activity` returns `Ok(())` (or logs at debug) for a missing user; no error-level logs for new users.
- Route/status table documented in the service README or docs/.
