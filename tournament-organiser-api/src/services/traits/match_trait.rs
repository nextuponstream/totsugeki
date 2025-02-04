//! Reusable match queries

use crate::repositories::matches::TournamentMatchRecord;
use crate::types::{SqlxError, SqlxTransaction};
use totsugeki_core::matches::Match;
use totsugeki_core::ID;

/// Reusable match queries
#[allow(dead_code)]
pub trait MatchTrait {
    /// Get matches of given `tournament_id`
    async fn get_matches_for_tournament(
        transaction: SqlxTransaction<'_, '_>,
        tournament_id: ID,
    ) -> Result<Vec<Match>, SqlxError> {
        let matches = sqlx::query_as!(
            TournamentMatchRecord,
            r#"
SELECT 
   id,
   tournament_id,
   match_id,
   pos,
   M.format,
   M.format_n,
   M.high_seed,
   M.high_seed_player,
   M.low_seed,
   M.low_seed_player
FROM tournament_matches
LEFT JOIN matches M on tournament_matches.match_id = M.id
WHERE tournament_id = $1
            "#,
            tournament_id
        )
        .fetch_all(&mut **transaction)
        .await?;
        Ok(matches.into_iter().map(std::convert::Into::into).collect())
    }
}
