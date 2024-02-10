//! health check endpoint

use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// Health check response
#[derive(Serialize, Deserialize, Debug)]
pub struct HealthCheck {
    /// true if api is health
    pub ok: bool,
}

/// `/health_check` endpoint for health check
pub(crate) async fn health_check() -> impl IntoResponse {
    Json(HealthCheck { ok: true })
}
