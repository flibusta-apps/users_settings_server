# Route status contract

Summary of current status codes for "not found" and donate-notifications routes.

| Route | Status codes |
| --- | --- |
| `GET /users/{user_id}` | `404` if user not found (previously `204`; standardized per spec 04). `200` `Json(UserDetail)` otherwise. |
| `POST /users/{user_id}/update_activity` | `404` if user not found. `200` otherwise. |
| `GET /languages/{code}` | `404` if not found (reference behavior, unchanged). |
| `GET /donate_notifications/{chat_id}/is_need_send?is_private=` | `200` `Json(bool)`. Unknown chat -> `true`. Non-atomic; reads then a separate `POST /donate_notifications/{chat_id}` writes, so it may race across concurrent callers/replicas — prefer `try_claim`. |
| `POST /donate_notifications/{chat_id}` | `200`. Marks sent (legacy, kept for backward compatibility). Use paired with `is_need_send` only from a single non-concurrent caller. |
| `POST /donate_notifications/{chat_id}/try_claim?is_private=` | `200` `Json(bool)`. Atomic check-and-mark in one SQL statement; returns `true` iff this call actually claimed the notification slot (fresh insert or the existing row was past its cooldown window), `false` if another caller already holds the slot. Recommended over the `is_need_send` + `mark_sent` pair to avoid duplicate-notification races. |

Note: `book_bot` has not yet migrated to `try_claim` as of this change (informational, no code change required here).
