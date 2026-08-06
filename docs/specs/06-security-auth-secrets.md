# Spec 06: Security — API key handling, committed credentials, secret-leaking script

- **Priority:** high
- **Effort:** S
- **Category:** security

## Problem(s)

### 06.1 Live database credentials sit in `.env` and leak into the Docker build context
`.env:1`:
```
DATABASE_URL=postgresql://flibusta_users_settings_user:<password>@kurbezz.me:54322/flibusta_users_settings
```
Real credentials for a publicly resolvable host/port. The file is gitignored (`.gitignore:1`) but there is **no `.dockerignore`**, and the builder does `COPY . .` (`docker/build.dockerfile:5`) — so `.env` (and `target/`, `.git` if present) enter the build context and the builder image layer. Worse, sqlx macros read `.env` at compile time: a local/CI `cargo build` will prepare queries against the *production* database instead of the `.sqlx` offline cache.

**Fix:** Rotate the exposed password. Add `.dockerignore` (`.env`, `target/`, `.git`, `.vscode`). Set `ENV SQLX_OFFLINE=true` in the builder stage so builds never touch a live DB.

### 06.2 `scripts/env.sh` prints all secrets to stdout
`scripts/env.sh:8`:
```sh
env | grep -E '^(API_KEY|POSTGRES_|SENTRY_DSN|APPLICATION_NAME)=' || true
```
Prints API key and DB password to stdout (i.e., container logs) if ever run. It is not invoked by `scripts/start.sh:3`, but `docker/build.dockerfile:18` copies `scripts/*.sh` into the image root, so it ships as an executable footgun. Dead code + secret exposure.

**Fix:** Delete `scripts/env.sh` and copy only `start.sh` in the Dockerfile (as services_manager_server already does).

### 06.3 API key compared with non-constant-time equality, no scheme
`src/views/mod.rs:35`: `if auth_header != CONFIG.api_key` — plain string comparison is theoretically timing-observable, and the raw `Authorization` header carries no scheme (`Bearer`), which confuses proxies/tooling. Low practical risk (single static key, internal network) but trivially hardened.

**Fix:** Use a constant-time comparison (e.g. `subtle::ConstantTimeEq` over byte slices after length check) and accept `Bearer <key>` while keeping the bare form for compatibility.

### 06.4 `/metrics` is unauthenticated
`src/views/mod.rs:59-60` mounts the Prometheus handle outside the `auth` middleware (applied only to `app_router`, line 55). Anyone who can reach the port can enumerate routes and traffic volumes. `/health` open is fine; metrics exposure is a (minor) information leak if the service is not network-isolated.

**Fix:** Either bind/scrape metrics on an internal-only port, protect with the API key, or document that deployment must not expose the port publicly.

## Acceptance criteria
- Exposed DB password rotated; `.dockerignore` present and `docker build` context contains no `.env`.
- Docker/CI builds succeed with `SQLX_OFFLINE=true` and no network access to the production DB.
- `scripts/env.sh` removed; image contains only `start.sh`.
- Auth accepts `Bearer <key>`, rejects wrong keys in constant time; `/metrics` not publicly reachable (or auth-protected).
