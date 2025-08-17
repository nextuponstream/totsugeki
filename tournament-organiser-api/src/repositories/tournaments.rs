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

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::PgPool;

    #[sqlx::test(fixtures("3_players_tournament.sql"))]
    async fn deleting_tournaments_deletes_related_data(db: PgPool) {
        let mut transaction = db.begin().await.unwrap();
        // TODO refactor to repository
        let tournaments = sqlx::query("SELECT * FROM tournaments")
            .fetch_all(&mut *transaction)
            .await
            .unwrap();
        assert_eq!(1, tournaments.len());
        // TODO refactor to repository
        let matches = sqlx::query("SELECT * FROM matches")
            .fetch_all(&mut *transaction)
            .await
            .unwrap();
        assert_eq!(5, matches.len());

        let tournament_id = "62aefc4c-d6ec-4c2f-98f0-b639688cbe0c".parse().unwrap();
        TournamentRepository::delete(&mut transaction, tournament_id)
            .await
            .unwrap();

        // TODO refactor to repository
        let tournaments = sqlx::query("SELECT * FROM tournaments")
            .fetch_all(&mut *transaction)
            .await
            .unwrap();
        assert_eq!(0, tournaments.len());
        // TODO refactor to repository
        let matches = sqlx::query("SELECT * FROM matches")
            .fetch_all(&mut *transaction)
            .await
            .unwrap();
        assert_eq!(0, matches.len());
        transaction.commit().await.unwrap();
    }
}
