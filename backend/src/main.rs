mod access;
mod auth;
mod basics;
mod config;
mod db;
mod error;
mod http;
mod routes;
mod state;

use config::AppConfig;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let config = AppConfig::load()?;
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url())
        .await?;
    let addr = config.server.socket_addr()?;

    let state = AppState { config, db };
    let app = routes::router()
        .layer(TraceLayer::new_for_http())
        .with_state(state);
    let listener = TcpListener::bind(addr).await?;

    info!(%addr, "dormtk backend listening");
    axum::serve(listener, app).await?;

    Ok(())
}

fn init_tracing() {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer())
        .init();
}
