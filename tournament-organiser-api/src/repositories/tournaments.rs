//! Persistence of tournaments

use crate::tournaments::Tournament;
use crate::types::{SqlxError, SqlxTransaction};
use crate::ID;
use std::fmt::Display;

/// Tournament repository
pub struct TournamentRepository {}

#[derive(sqlx::Type)]
#[sqlx(type_name = "format")]
#[allow(
    non_camel_case_types,
    reason = "serializing postgres enum into sqlx enum"
)]
pub enum Format {
    double_elimination,
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Format::double_elimination => "double_elimination",
            }
        )
    }
}

/// Tournament from database
pub struct TournamentRecord {
    /// ID
    pub(crate) id: ID,
    /// Name
    pub(crate) name: String,
    /// Format
    pub(crate) format: Format,
}

impl TournamentRepository {
    /// Create tournament
    pub(crate) async fn create(
        transaction: SqlxTransaction<'_, '_>,
        tournament: &Tournament,
    ) -> Result<(), SqlxError> {
        let _ = sqlx::query!(
            "INSERT INTO tournaments (id, name, format) VALUES ($1, $2, $3)",
            tournament.get_id(),
            tournament.get_name(),
            "double_elimination".to_string(), // FIXME allow other formats
        )
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }

    pub(crate) async fn all(
        transaction: SqlxTransaction<'_, '_>,
    ) -> Result<Vec<TournamentRecord>, SqlxError> {
        let tuples = sqlx::query_as!(
            TournamentRecord,
            r#"
SELECT
    id,
    name,
    format AS "format!: Format"
from tournaments
"#,
        )
        .fetch_all(&mut **transaction)
        .await?;
        Ok(tuples.into_iter().collect())
    }

    /// Delete tournament
    pub async fn delete(transaction: SqlxTransaction<'_, '_>, id: ID) -> Result<(), SqlxError> {
        let _ = sqlx::query!("DELETE FROM tournaments WHERE id = $1", id)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }
}
