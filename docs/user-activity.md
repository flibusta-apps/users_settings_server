# `user_activity` table

`user_activity` is written by `POST /users/{user_id}/update_activity` (see
`src/views/users/mod.rs`), but there is no corresponding read endpoint
exposed by this service.

This is intentional: the table exists for ad-hoc SQL analytics and
observability queries run directly against the database, not for API
consumption. No code or schema change is planned for this — if a read
use case emerges, it should be added as a new endpoint rather than assumed
to already exist.
