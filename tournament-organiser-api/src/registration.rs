//! registration

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use chrono::prelude::*;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use totsugeki::player::Id;

/// Standard error message
#[derive(Serialize, Deserialize)]
pub struct ErrorResponse {
    /// user-facing error message
    pub message: String,
}

/// User registration form input
#[derive(Serialize, Deserialize, Debug)]
pub struct FormInput {
    /// user name
    pub name: String,
    /// user email address
    pub email: String,
    /// user provided password
    pub password: String,
    /// user id
    pub created_at: Option<String>,
}

/// User of application
#[derive(sqlx::FromRow)]
// reason = will be used later
#[allow(dead_code)]
struct User {
    /// Id of user
    pub id: Id,
    /// user name
    pub name: String,
    /// user email address
    pub email: String,
    /// user id
    pub created_at: Option<DateTime<Utc>>,
}

/// `/register` endpoint for health check
pub(crate) async fn user_registration(
    State(pool): State<PgPool>,
    Json(form_input): Json<FormInput>,
) -> impl IntoResponse {
    tracing::debug!("new user registration");
    if sqlx::query_as!(
        User,
        "SELECT * from users WHERE email = $1",
        &form_input.email,
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_optional(&pool)
    .await
    .expect("select first user with matching email")
    .is_some()
    {
        let message = "Another user has already registered with provided mail".to_string();
        tracing::warn!(message);
        return (StatusCode::CONFLICT, Json(ErrorResponse { message })).into_response();
    }

    let _r = sqlx::query!(
        "INSERT INTO users (name, email) VALUES ($1, $2)",
        form_input.name,
        form_input.email
    )
    .execute(&pool)
    .await
    .expect("user insert");
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    tracing::info!("new user {}", form_input.email);

    (StatusCode::OK, Json(())).into_response()
}
