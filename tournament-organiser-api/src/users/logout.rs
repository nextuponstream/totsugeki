//! Logout handling

use axum::response::IntoResponse;
use http::StatusCode;
use tower_sessions::Session;
use tracing::instrument;

/// `/logout` endpoint
#[instrument(name = "logout", skip(session))]
pub(crate) async fn logout(session: Session) -> impl IntoResponse {
    session.clear().await; // no user_id to pick from
                           // if you log back in, there won't be a warning like:
                           // call:login:get_record: tower_sessions_core::session: possibly suspicious activity: record not found in store
    session.save().await.expect("logged out user");
    tracing::info!("successful logout");
    (StatusCode::OK).into_response()
}
