//! user actions
use crate::registration::User;
use axum::extract::State;
use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use totsugeki::player::Id;
use tower_sessions::Session;
use tracing::instrument;

/// User retrieving his infos
#[derive(Serialize, Deserialize, Debug)]
pub struct Infos {
    /// user email
    pub email: String,
    /// username
    pub name: String,
}

/// `/api/user/profile` GET to check user informations
#[instrument(name = "user_dashboard", skip(pool, session))]
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

/// `/api/user` DELETE
#[instrument(name = "user_account_deletion", skip(pool, session))]
pub(crate) async fn delete_user(session: Session, State(pool): State<PgPool>) -> impl IntoResponse {
    let Some(Some(user_id)): Option<Option<Id>> = session
        .get("user_id")
        .await
        .expect("session store maybe value")
    else {
        tracing::warn!("missing session");
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    let row = match sqlx::query!("SELECT email from users WHERE id = $1", user_id,)
        .fetch_optional(&pool)
        .await
    {
        Ok(r) => r,
        Err(e) => {
            tracing::error!("user row: {e}");
            panic!("user row: {e}");
        }
    };
    let email = match row {
        Some(r) => r.email,
        None => return (StatusCode::NOT_FOUND).into_response(),
    };

    let Ok(r) = sqlx::query!("DELETE from users WHERE id = $1", user_id)
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .execute(&pool)
        .await
    else {
        tracing::error!("User {} could not delete their account", user_id);
        return (StatusCode::NOT_FOUND).into_response();
    };

    if r.rows_affected() != 1 {
        #[allow(clippy::uninlined_format_args)]
        let err_msg = format!(
            "{} User deleted more than one user when deleting their account",
            user_id
        );
        tracing::error!(err_msg);
        unreachable!("{err_msg}")
    }
    session.delete().await.expect("deleted session");
    tracing::info!("{} deleted their account ({})", email, user_id);

    (StatusCode::OK).into_response()
}
