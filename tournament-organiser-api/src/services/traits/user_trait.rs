//! Reusable user queries

use crate::tournaments::TournamentID;
use crate::types::{SqlxError, SqlxTransaction};
use crate::users::registration::UserID;

/// UserTrait
pub trait UserTrait {
    /// True if user `user_id` is tournament organiser of `tournament_id`
    async fn is_tournament_organiser<'a>(
        transaction: SqlxTransaction<'a, '_>,
        user_id: UserID,
        tournament_id: TournamentID,
    ) -> Result<bool, SqlxError> {
        let is_tournament_organiser = sqlx::query!(
            r#"SELECT tournament_id, user_id 
            FROM tournament_organisers
            WHERE user_id = $1
            AND tournament_id = $2
            "#,
            user_id.0,
            tournament_id.get()
        )
        .fetch_optional(&mut **transaction)
        .await?
        .is_some();

        Ok(is_tournament_organiser)
    }
}
