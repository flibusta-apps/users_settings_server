# Spec 07: Delivery — root container, no healthcheck, CI gates nothing

- **Priority:** medium
- **Effort:** M
- **Category:** delivery

## Problem(s)

### 07.1 Container runs as root with no HEALTHCHECK
`docker/build.dockerfile:10-24`: the runtime stage never adds a `USER`, so the service runs as root; there is no `HEALTHCHECK` despite the app exposing `GET /health` (`src/views/mod.rs:62`). The image also lacks `curl`/`wget`, so an orchestrator-side exec healthcheck has nothing to call with.

**Fix:** Add a non-root user (`useradd -r app` + `USER app`), and `HEALTHCHECK CMD curl -f http://localhost:8080/health || exit 1` (installing curl, as the services_manager image already does) or rely on orchestrator HTTP checks — but document it.

### 07.2 Docker build has no dependency caching and builds from a dirty context
`docker/build.dockerfile:5-7`: `COPY . .` then `cargo build --release` — every code change invalidates the whole dependency build (~full recompile per CI run), and the context includes `.env`/`target` (see Spec 06.1, no `.dockerignore`).

**Fix:** Add `.dockerignore`; use cargo layer caching (copy `Cargo.toml`/`Cargo.lock` + dummy `main.rs`, build deps, then copy `src/`), or `cargo-chef`.

### 07.3 CI never runs tests or a blocking lint; deploys on every push to main
- `.github/workflows/build_docker_image.yaml` builds and pushes `:latest` + triggers the deployment webhook (lines 30-45) with no preceding test/check job.
- `.github/workflows/rust-clippy.yml:49`: `continue-on-error: true` — clippy failures never block anything; there is no `cargo test` / `cargo build` gating workflow at all.
- There are no tests in the repository (`src/` contains zero `#[test]`/`tests/` items) — combined with Specs 01-03 this is how the `page=0` and ARRAY_AGG bugs survive.

**Fix:** Add a CI job running `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test` (with `SQLX_OFFLINE=true`; integration tests against a `services: postgres` container running the migrations), and make the Docker build/deploy job depend on it.

### 07.4 Hardcoded bind address and no graceful shutdown
`src/main.rs:15`: `SocketAddr::from(([0, 0, 0, 0], 8080))` — port not configurable; `axum::serve(listener, app)` (main.rs:19) has no `with_graceful_shutdown`, so deploys drop in-flight requests.

**Fix:** Read `PORT` env (default 8080); add SIGTERM/SIGINT graceful shutdown handler.

## Acceptance criteria
- `docker inspect` shows non-root user and a healthcheck (or documented orchestrator probe).
- Rebuilding after a src-only change reuses the cached dependency layer.
- CI: a failing test or clippy warning blocks the image build/deploy job.
- At least smoke-level integration tests exist (fresh Postgres + migrations + CRUD round-trip).
- SIGTERM results in in-flight requests completing before exit.
