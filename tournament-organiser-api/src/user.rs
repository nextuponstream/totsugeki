//! user actions
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

/// User retrieving his infos
#[derive(Serialize, Deserialize, Debug)]
pub struct Infos {
    /// user email
    pub email: String,
    /// username
    pub name: String,
}

/// `/api/user/profile` to check user informations
pub(crate) async fn profile() -> impl IntoResponse {
    Json(Infos {
        email: todo!(),
        name: todo!(),
    })
}
