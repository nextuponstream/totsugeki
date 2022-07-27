//! Healthcheck Api

use poem_openapi::{payload::PlainText, OpenApi};

/// Healthcheck Api
pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/health_check", method = "get")]
    /// monitoring endpoint
    async fn health_check(&self) -> PlainText<String> {
        PlainText("Ok".to_string())
    }
}
