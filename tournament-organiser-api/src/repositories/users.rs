//! User repository

use crate::users::registration::UserRecord;
use sqlx::error::Error as SqlxError;
use sqlx::{Postgres, Transaction};
use totsugeki::player::Id;

pub(crate) struct UserRepository {}
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
    pub async fn read(
        transaction: &mut Transaction<'_, Postgres>,
        user_id: Id,
    ) -> Result<Option<UserRecord>, Error> {
        let u = sqlx::query_as!(
            UserRecord,
            r#"SELECT id, name, email from users WHERE id = $1"#,
            user_id
        )
        .fetch_optional(&mut **transaction)
        .await?;
        Ok(u)
    }
}
