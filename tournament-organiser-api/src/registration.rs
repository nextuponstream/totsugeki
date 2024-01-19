//! registration

use axum::extract::State;
use axum::response::{IntoResponse, Json};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPool;
use totsugeki::player::Id;

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
    if sqlx::query_as::<_, User>("SELECT * from users WHERE email = $1")
        .bind(&form_input.email)
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_optional(&pool)
        .await
        .expect("select first user with matching email")
        .is_some()
    {
        // return error already exist
        todo!("User already exists")
    } else {
        // FIXME no rows inserted
        let _r = sqlx::query("INSERT INTO users (name, email) VALUES ($1, $2)")
            .bind(form_input.name)
            .bind(form_input.email)
            .execute(&pool)
            .await
            .expect("user insert");
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
    }
    Json(())
}
