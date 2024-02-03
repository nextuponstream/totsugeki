//! Logout handling

use axum::response::IntoResponse;
use http::StatusCode;
use tower_sessions::Session;
use tracing::instrument;

/// `/logout` endpoint
#[instrument(name = "logout", skip(session))]
pub(crate) async fn logout(session: Session) -> impl IntoResponse {
    session.delete().await.expect("deleted session");
    tracing::info!("successful logout");
    (StatusCode::OK).into_response()
}
