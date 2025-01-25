//! registration

use crate::http::{Error, ErrorSlug};
use crate::repositories::users::UserRepository;
use crate::ApiResponse;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use argon2::PasswordHasher;
use axum::extract::State;
use axum::response::{IntoResponse, Json};
use http::StatusCode;
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use sqlx::{Postgres, Transaction};
use totsugeki_core::ID;
use tracing::instrument;
use uuid::Uuid;
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
    pub password: SecretString,
    /// user id
    pub created_at: Option<String>,
}

/// User of application
#[derive(sqlx::Type, Serialize, Clone, Debug, Copy, Deserialize)]
pub struct UserID(pub ID);

impl From<Uuid> for UserID {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

/// User of application
#[derive(sqlx::FromRow, Clone, Debug)]
pub struct User {
    /// Id of user
    pub id: UserID,
    /// user name
    pub name: String,
    /// user email address
    pub email: String,
    /// user password hash
    pub password: String,
    /// user id
    #[allow(dead_code)]
    pub created_at: Option<OffsetDateTime>,
}

/// User of application
#[derive(sqlx::FromRow, Clone, Debug)]
pub struct UserRecord {
    /// Id of user
    pub id: UserID,
    /// user name
    pub name: String,
    /// user email address
    pub email: String,
}

async fn get_transaction_from_pool<'a>(
    pg_pool: PgPool,
) -> Result<Transaction<'a, Postgres>, ErrorSlug> {
    Ok(pg_pool
        .begin()
        .await
        .map_err(|_| ErrorSlug::new(StatusCode::INTERNAL_SERVER_ERROR, "sqlx".to_string()))?)
}

use axum::debug_handler;
use time::OffsetDateTime;

/// `/register` endpoint for health check
#[instrument(name = "user_registration", skip(pool))]
#[debug_handler]
pub(crate) async fn registration(
    State(pool): State<PgPool>,
    Json(form_input): Json<FormInput>,
) -> impl IntoResponse {
    let mut transaction = pool.begin().await?;
    if UserRepository::exists(&mut transaction, &form_input.email).await {
        let message = "Another user has already registered with provided mail".to_string();
        tracing::warn!(message);
        return Ok((StatusCode::CONFLICT, Json(ApiResponse { message })));
    }

    let raw_password = form_input.password.expose_secret();

    let estimate =
        zxcvbn(raw_password, &[&form_input.name, &form_input.email]).expect("password analysis");
    if let Some(feedback) = estimate.feedback() {
        if let Some(warning) = feedback.warning() {
            // NOTE: might contain password attempt if you log feedback
            return Ok((
                StatusCode::BAD_REQUEST,
                Json(ApiResponse {
                    #[allow(clippy::uninlined_format_args)]
                    message: format!("weak_password: {}", warning),
                }),
            ));
        }

        return Ok((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse {
                message: "weak_password".into(),
            }),
        ));
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
    .await?;
    // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    tracing::info!("new user {}", form_input.email);

    // Ok::<(StatusCode, AxumJson<()>), Error>((StatusCode::OK, Json(())))
    // TODO make it a type
    // StatusCodeAndMessage::Ok((StatusCode::CREATED, Json(ApiResponse::default())))
    // Ok::<_, Error>((StatusCode::CREATED, Json(ApiResponse::default())))
    Ok::<(StatusCode, Json<ApiResponse>), Error>((
        StatusCode::CREATED,
        Json(ApiResponse::default()),
    ))
}
