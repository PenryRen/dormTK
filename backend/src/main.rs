mod config;
mod state;

use axum::Router;
use config::AppConfig;
use sqlx::postgres::PgPoolOptions;
use state::AppState;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppConfig::load()?;
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&config.database.url())
        .await?;
    let addr = config.server.socket_addr()?;

    let state = AppState { config, db };
    let app = Router::new().with_state(state);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
