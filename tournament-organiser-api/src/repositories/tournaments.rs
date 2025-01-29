//! Persistence of tournaments

use crate::tournaments::Tournament;
use crate::types::{SqlxError, SqlxTransaction};

/// Tournament repository
pub struct TournamentRepository {}

impl TournamentRepository {
    /// Create tournament
    pub async fn create(
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
}
