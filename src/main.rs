pub mod config;
pub mod prisma;
pub mod views;
pub mod db;

use sentry::{ClientOptions, types::Dsn, integrations::debug_images::DebugImagesIntegration};
use tracing::info;

use std::{net::SocketAddr, str::FromStr};


async fn start_app() {
    let app = views::get_router().await;

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    info!("Start webserver...");
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
    info!("Webserver shutdown...")
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let options = ClientOptions {
        dsn: Some(Dsn::from_str(&config::CONFIG.sentry_dsn).unwrap()),
        default_integrations: false,
        ..Default::default()
    }
    .add_integration(DebugImagesIntegration::new());

    let _guard = sentry::init(options);

    start_app().await;
}
