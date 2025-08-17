//! Persistence of tournaments

use crate::tournaments::Tournament;
use crate::types::{SqlxError, SqlxTransaction};
use crate::ID;

/// Tournament repository
pub struct TournamentRepository {}

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

    /// Delete tournament
    pub async fn delete(transaction: SqlxTransaction<'_, '_>, id: ID) -> Result<(), SqlxError> {
        let _ = sqlx::query!("DELETE FROM tournaments WHERE id = $1", id)
            .execute(&mut **transaction)
            .await?;
        Ok(())
    }
}
