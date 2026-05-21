use crate::{access, basics, duties, error::ApiError, state::AppState};
use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(access::routes())
        .merge(basics::routes())
        .merge(duties::routes())
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

async fn health(State(state): State<AppState>) -> Result<Json<HealthResponse>, ApiError> {
    let _server_port = state.config.server.port;
    let _connection = state.db.acquire().await?;

    Ok(Json(HealthResponse { status: "ok" }))
}
