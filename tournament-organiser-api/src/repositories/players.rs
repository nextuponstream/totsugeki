//! User participating in tournaments

use crate::tournaments::TournamentID;
use crate::types::{SqlxError, SqlxTransaction};
use crate::users::registration::UserID;

pub struct PlayerRepository {}

impl PlayerRepository {
    pub async fn is_user_a_player_in_tournament<'a>(
        transaction: SqlxTransaction<'a, '_>,
        user_id: UserID,
        tournament_id: TournamentID,
    ) -> Result<bool, SqlxError> {
        let user_is_player_of_tournament = sqlx::query!(
            r#"SELECT tournament_id, player_id FROM players WHERE player_id = $1"#,
            user_id.0
        )
        .fetch_optional(&mut **transaction)
        .await?
        .is_some();

        Ok(user_is_player_of_tournament)
    }
}
