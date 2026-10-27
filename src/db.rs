use crate::config::CONFIG;

use sqlx::{
    postgres::{PgConnectOptions, PgPoolOptions},
    PgPool,
};
use tracing::info;

fn base_connect_options() -> PgConnectOptions {
    PgConnectOptions::new()
        .host(&CONFIG.postgres_host)
        .port(CONFIG.postgres_port)
        .username(&CONFIG.postgres_user)
        .password(&CONFIG.postgres_password)
        .database(&CONFIG.postgres_db)
        .application_name(&CONFIG.application_name)
}

pub async fn get_postgres_pool() -> PgPool {
    info!(
        "Creating Postgres pool (app_name={}, max_connections={}, min_connections={}, acquire_timeout_sec={})",
        CONFIG.application_name,
        CONFIG.postgres_pool_max_connections,
        CONFIG.postgres_pool_min_connections,
        CONFIG.postgres_pool_acquire_timeout_sec
    );

    // Migrations run on a dedicated connection without statement/lock timeouts:
    // they may legitimately take longer than request-level limits.
    {
        let migration_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect_with(base_connect_options())
            .await
            .expect("Failed to connect to Postgres");

        info!("Running database migrations...");
        sqlx::migrate!("./migrations")
            .run(&migration_pool)
            .await
            .expect("Failed to run migrations");
        info!("Database migrations completed successfully");

        migration_pool.close().await;
    }

    let statement_timeout = CONFIG.postgres_statement_timeout_ms.to_string();
    let lock_timeout = CONFIG.postgres_lock_timeout_ms.to_string();
    let options = base_connect_options().options([
        ("statement_timeout", statement_timeout.as_str()),
        ("lock_timeout", lock_timeout.as_str()),
    ]);

    PgPoolOptions::new()
        .max_connections(CONFIG.postgres_pool_max_connections)
        .min_connections(CONFIG.postgres_pool_min_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            CONFIG.postgres_pool_acquire_timeout_sec,
        ))
        .connect_with(options)
        .await
        .expect("Failed to connect to Postgres")
}
