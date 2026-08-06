# users_settings_server — Audit Specs Index

Audit date: 2026-07-07. Scope: `src/`, `Cargo.toml`, `migrations/`, `docker/`, `scripts/`, `.github/`, plus the book_bot consumer contract.

| Spec | Title | Priority | Effort | Category |
|------|-------|----------|--------|----------|
| [01](01-panic-driven-error-handling.md) | Panic-driven error handling crashes the whole process | high | M | reliability |
| [02](02-migrations-schema-gaps.md) | Migrations do not create everything the code depends on | high | M | correctness |
| [03](03-user-languages-sync-bugs.md) | User languages sync — duplicates, no transaction, NULL-row decode panic | high | M | correctness |
| [04](04-not-found-status-contract.md) | Inconsistent "user not found" status codes (204 vs 404) | medium | S | correctness |
| [05](05-donate-notifications-logic.md) | Donate notifications — non-atomic check/mark, fragile time handling | medium | M | correctness |
| [06](06-security-auth-secrets.md) | Security — API key handling, committed credentials, secret-leaking script | high | S | security |
| [07](07-delivery-docker-ci.md) | Delivery — root container, no healthcheck, CI gates nothing | medium | M | delivery |
| [08](08-observability-ops.md) | Observability and operational hardening | low | S | observability |
| [09](09-performance-hot-path.md) | Hot-path performance — missing JOIN index, pool defaults, request fan-out | high | S-M | performance |

Key contract note: for a nonexistent user, `GET /users/{id}` returns **204 No Content** (src/views/users/mod.rs:100-102) while `POST /users/{id}/update_activity` returns **404 Not Found** (src/views/users/mod.rs:189) — see Spec 04.

Suggested order: 06.1 (rotate creds) → 01 → 02 → 03 → 04/05 → 07 → 08.
