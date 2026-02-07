use crate::config::CONFIG;

use sqlx::{postgres::PgPoolOptions, PgPool};
use tracing::info;

pub async fn get_postgres_pool() -> PgPool {
    let database_url: String = format!(
        "postgresql://{}:{}@{}:{}/{}?application_name={}",
        CONFIG.postgres_user,
        CONFIG.postgres_password,
        CONFIG.postgres_host,
        CONFIG.postgres_port,
        CONFIG.postgres_db,
        CONFIG.application_name
    );

    info!(
        "Creating Postgres pool (app_name={}, max_connections={}, acquire_timeout_sec={})",
        CONFIG.application_name,
        CONFIG.postgres_pool_max_connections,
        CONFIG.postgres_pool_acquire_timeout_sec
    );

    let pool = PgPoolOptions::new()
        .max_connections(CONFIG.postgres_pool_max_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            CONFIG.postgres_pool_acquire_timeout_sec,
        ))
        .connect(&database_url)
        .await
        .unwrap();

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");
    info!("Database migrations completed successfully");

    pool
}
