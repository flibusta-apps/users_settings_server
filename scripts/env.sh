#! /usr/bin/env sh

# Environment variables should be provided directly (e.g. via docker-compose, k8s, etc.)
# Required: API_KEY, POSTGRES_USER, POSTGRES_PASSWORD, POSTGRES_HOST, POSTGRES_PORT,
#           POSTGRES_DB, SENTRY_DSN
# Optional: POSTGRES_POOL_MAX_CONNECTIONS, POSTGRES_POOL_ACQUIRE_TIMEOUT_SEC, APPLICATION_NAME

env | grep -E '^(API_KEY|POSTGRES_|SENTRY_DSN|APPLICATION_NAME)=' || true
