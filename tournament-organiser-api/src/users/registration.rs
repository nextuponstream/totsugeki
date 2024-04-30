//! registration

use crate::ErrorResponse;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use axum::extract::State;
use axum::response::{IntoResponse, Json};
use chrono::prelude::*;
use http::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use sqlx::postgres::PgPool;
use totsugeki::player::Id;
use tracing::instrument;
use zxcvbn::zxcvbn;

/// User registration form input with secret input to avoid it being exposed
/// through logs
#[derive(Deserialize, Debug)]
pub struct FormInput {
    /// user name
    pub name: String,
    /// user email address
    pub email: String,
    /// user provided password
    pub password: Secret<String>,
    /// user id
    pub created_at: Option<String>,
}

/// User of application
#[derive(sqlx::FromRow, Clone, Debug)]
pub struct User {
    /// Id of user
    pub id: Id,
    /// user name
    pub name: String,
    /// user email address
    pub email: String,
    /// user password hash
    pub password: String,
    /// user id
    #[allow(dead_code)]
    pub created_at: Option<DateTime<Utc>>,
}

/// `/register` endpoint for health check
#[instrument(name = "user_registration", skip(pool))]
pub(crate) async fn registration(
    State(pool): State<PgPool>,
    Json(form_input): Json<FormInput>,
) -> impl IntoResponse {
    if sqlx::query_as!(
        User,
        "SELECT * from users WHERE email = $1",
        &form_input.email,
    )
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    .fetch_optional(&pool)
    .await
    .expect("user with matching email")
    .is_some()
    {
        let message = "Another user has already registered with provided mail".to_string();
        tracing::warn!(message);
        return (StatusCode::CONFLICT, Json(ErrorResponse { message })).into_response();
    }

    let raw_password = form_input.password.expose_secret();

    let estimate =
        zxcvbn(raw_password, &[&form_input.name, &form_input.email]).expect("password analysis");
    if let Some(feedback) = estimate.feedback() {
        if let Some(warning) = feedback.warning() {
            // NOTE: might contain password attempt if you log feedback
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    #[allow(clippy::uninlined_format_args)]
                    message: format!("weak_password: {}", warning),
                }),
            )
                .into_response();
        }

        return (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                message: "weak_password".into(),
            }),
        )
            .into_response();
    };

    // Copied from zero2prod book
    let salt = SaltString::generate(&mut rand::thread_rng());
    let password_hash = Argon2::default()
        .hash_password(raw_password.as_bytes(), &salt)
        .expect("password in PHC format")
        .to_string();

    let _r = sqlx::query!(
        "INSERT INTO users (name, email, password) VALUES ($1, $2, $3)",
        form_input.name,
        form_input.email,
        password_hash,
    )
    .execute(&pool)
    .await
    .expect("user insert");
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    tracing::info!("new user {}", form_input.email);

    (StatusCode::OK, Json(())).into_response()
}
