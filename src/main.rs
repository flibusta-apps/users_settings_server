pub mod config;
pub mod prisma;
pub mod views;
pub mod db;

use std::net::SocketAddr;


async fn start_app() {
    let app = views::get_router();

    // env_logger::init();

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    log::info!("Start webserver...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    log::info!("Webserver shutdown...")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let _guard = sentry::init(config::CONFIG.sentry_dsn.clone());

    start_app().await;
}
