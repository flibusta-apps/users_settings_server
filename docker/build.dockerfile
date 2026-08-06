FROM rust:bookworm AS builder

ENV SQLX_OFFLINE=true

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release --bin users_settings_server \
    && rm -rf src

COPY . .

RUN rm -rf target/release/users_settings_server target/release/deps/users_settings_server* \
    && cargo build --release --bin users_settings_server


FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y openssl ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

RUN useradd -r -u 1000 app

COPY ./scripts/start.sh /
RUN chmod +x /start.sh

WORKDIR /app

COPY --from=builder /app/target/release/users_settings_server /usr/local/bin
RUN chown app:app /usr/local/bin/users_settings_server

USER app

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/ready || exit 1

CMD ["/start.sh"]
