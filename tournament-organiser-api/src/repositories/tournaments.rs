//! Persistence of tournaments

use crate::tournaments::Tournament;
use crate::types::{SqlxError, SqlxTransaction};

/// Tournament repository
pub struct TournamentRepository {}

impl TournamentRepository {
    /// Create tournament
    pub async fn create<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament: &Tournament,
    ) -> Result<(), SqlxError> {
        let _ = sqlx::query!(
            "INSERT INTO tournaments (id, name, format) VALUES ($1, $2, $3)",
            tournament.get_id().get(), // FIXME this is not good syntax
            tournament.get_name(),
            "double_elimination".to_string(), // FIXME allow other formats
        )
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }
}
