//! Login user with their credentials

use argon2::{PasswordHash, PasswordVerifier};
use axum::body::Body;
use axum::extract::State;
use axum::http::header::AUTHORIZATION;
use axum::http::Request;
use axum::response::{IntoResponse, Json};
use base64::Engine;
use http::StatusCode;
use secrecy::{ExposeSecret, Secret};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use totsugeki::player::Id as UserId;
use tracing::instrument;

/// Credentials used by user
#[derive(Debug, Deserialize)]
pub(crate) struct Credentials {
    /// user email
    email: String,
    /// user password
    password: Secret<String>,
}

/// Successful login response with ID of logged in user
#[derive(Serialize)]
struct SuccessfulLogin {
    /// User ID
    user_id: UserId,
}

/// `/register` endpoint for health check
#[instrument(name = "login", skip(pool))]
pub(crate) async fn login(State(pool): State<PgPool>, request: Request<Body>) -> impl IntoResponse {
    let Some(authorization_header) = request.headers().get(AUTHORIZATION) else {
        panic!("Missing AUTHORIZATION header")
    };
    let base64encoded_segment = authorization_header
        .to_str()
        .unwrap()
        .strip_prefix("Basic ")
        .unwrap();
    let decoded_bytes = base64::engine::general_purpose::STANDARD.decode(base64encoded_segment);
    let decoded_credentials = String::from_utf8(decoded_bytes.unwrap()).unwrap();
    let mut credentials = decoded_credentials.splitn(2, ':');
    let email = credentials.next().unwrap().to_string();
    let password = Secret::new(credentials.next().unwrap().to_string());
    let credentials = Credentials { email, password };
    let row = sqlx::query!(
        "SELECT id, password from users WHERE email = $1",
        credentials.email,
    )
    .fetch_optional(&pool)
    .await
    .expect("potential user");
    let (user_id, password) = match row {
        Some(r) => (r.id, r.password),
        None => todo!(),
    };
    // FIXME use fixed params with new constructor rather than rely on defaults
    // that may change
    let hasher = argon2::Argon2::default();
    let expected_hash = PasswordHash::new(&password).expect("password in PHC format");
    let Ok(()) = hasher.verify_password(
        credentials.password.expose_secret().as_bytes(),
        &expected_hash,
    ) else {
        return (StatusCode::UNAUTHORIZED).into_response();
    };
    tracing::info!("successful login");
    (StatusCode::OK, Json(SuccessfulLogin { user_id })).into_response()
}
