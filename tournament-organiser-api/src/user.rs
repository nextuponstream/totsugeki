//! user actions
use crate::registration::User;
use axum::extract::State;
use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use totsugeki::player::Id;
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
pub(crate) async fn profile(session: Session, State(pool): State<PgPool>) -> impl IntoResponse {
    let Some(Some(user_id)): Option<Option<Id>> = session
        .get("user_id")
        .await
        .expect("session store maybe value")
    else {
        tracing::warn!("profile was not displayed because user is not logged in");
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    tracing::debug!("{:?}", user_id);
    let Some(u) = sqlx::query_as!(User, "SELECT * from users WHERE id = $1", user_id,)
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_optional(&pool)
        .await
        .expect("fetch result")
    else {
        return (StatusCode::NOT_FOUND).into_response();
    };
    Json(Infos {
        email: u.email,
        name: u.name,
    })
    .into_response()
}
