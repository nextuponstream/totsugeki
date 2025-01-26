//! Reusable match queries

use crate::repositories::matches::TournamentMatchRecord;
use crate::types::{SqlxError, SqlxTransaction};
use totsugeki_core::matches::Match;
use totsugeki_core::ID;

/// Reusable match queries
pub trait MatchTrait {
    /// Get matches of given `tournament_id`
    async fn get_matches_for_tournament<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: ID,
    ) -> Result<Vec<Match>, SqlxError> {
        let matches = sqlx::query_as!(
            TournamentMatchRecord,
            r#"
SELECT * from tournament_matches
LEFT JOIN matches on tournament_matches.match_id = matches.id
WHERE tournament_id = $1
            "#,
            tournament_id
        )
        .fetch_all(&mut **transaction)
        .await?;
        Ok(matches.into_iter().map(|v| v.into()).collect())
    }
}
