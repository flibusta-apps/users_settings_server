FROM rust:bookworm AS builder

ENV SQLX_OFFLINE=true

WORKDIR /app

COPY . .

RUN cargo build --release --bin users_settings_server


FROM debian:bookworm-slim

RUN apt-get update \
    && apt-get install -y openssl ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN update-ca-certificates

COPY ./scripts/start.sh /
RUN chmod +x /start.sh

WORKDIR /app

COPY --from=builder /app/target/release/users_settings_server /usr/local/bin
CMD ["/start.sh"]
