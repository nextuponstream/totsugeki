//! User repository

use crate::types::SqlxTransaction;
use crate::users::registration::{User, UserID, UserRecord};
use sqlx::error::Error as SqlxError;

/// All methods to query user in database
pub(crate) struct UserRepository {}

/// Errors while reading user in database
#[derive(Debug)]
pub(crate) enum Error {
    #[allow(dead_code)]
    /// Error with sqlx, unrecoverable
    Sqlx(SqlxError),
}
impl From<SqlxError> for Error {
    fn from(err: SqlxError) -> Self {
        Self::Sqlx(err)
    }
}

impl UserRepository {
    /// Read user from database
    pub async fn read<'a>(
        transaction: SqlxTransaction<'a, '_>,
        user_id: UserID,
    ) -> Result<Option<UserRecord>, Error> {
        let u = sqlx::query_as!(
            UserRecord,
            r#"SELECT id, name, email from users WHERE id = $1"#,
            user_id.0
        )
        .fetch_optional(&mut **transaction)
        .await?;
        Ok(u)
    }

    /// Returns `true` if user with given `email` exists
    pub async fn exists<'a>(transaction: SqlxTransaction<'a, '_>, email: &str) -> bool {
        sqlx::query_as!(User, "SELECT * from users WHERE email = $1", email,)
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
            .fetch_optional(&mut **transaction)
            .await
            .expect("user with matching email")
            .is_some()
    }
}
