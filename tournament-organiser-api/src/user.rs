//! user actions
use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use tower_sessions::Session;

/// User retrieving his infos
#[derive(Serialize, Deserialize, Debug)]
pub struct Infos {
    /// user email
    pub email: String,
    /// username
    pub name: String,
}

/// `/api/user/profile` to check user informations
pub(crate) async fn profile(session: Session) -> impl IntoResponse {
    let Some(user_id) = session.get_value("user_id").await.unwrap() else {
        tracing::warn!("profile was not displayed because user is not logged in");
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    tracing::info!("{:?}", user_id);
    Json(Infos {
        email: "".into(),
        name: user_id.to_string(),
    })
    .into_response()
}
