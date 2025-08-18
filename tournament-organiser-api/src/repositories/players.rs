//! Persist players of tournament

use crate::guests::Guest;
use crate::tournaments::tournament_players::TournamentPlayer;
use crate::tournaments::PlayerRecord;
use crate::types::{SqlxError, SqlxTransaction};
use crate::ID;
use bigdecimal::ToPrimitive;

/// Persist players of tournament
pub struct PlayerRepository {}

impl PlayerRepository {
    /// Returns `true` if `user_id` is a player of `tournament_id`
    pub async fn is_user_a_player_in_tournament(
        transaction: SqlxTransaction<'_, '_>,
        user_id: ID,
        tournament_id: ID,
    ) -> Result<bool, SqlxError> {
        let user_is_player_of_tournament = sqlx::query!(
            r#"
SELECT tournament_id, user_id FROM players
WHERE user_id = $1
AND tournament_id = $2
            "#,
            user_id,
            tournament_id
        )
        .fetch_optional(&mut **transaction)
        .await?
        .is_some();

        Ok(user_is_player_of_tournament)
    }

    /// Add guests as tournament players
    pub async fn add_guests(
        transaction: SqlxTransaction<'_, '_>,
        tournament_id: ID,
        guests: Vec<Guest>,
    ) -> Result<Vec<TournamentPlayer>, SqlxError> {
        let mut players = vec![];
        for (index, guest) in guests.iter().enumerate() {
            let player_id = sqlx::query!(
                r#"
INSERT INTO players (tournament_id, guest_id, seeding_index) VALUEs ($1, $2, $3) RETURNING id;
              "#,
                tournament_id,
                guest.get_id(),
                (index + 1).to_i16().expect("type coercion"),
            )
            .fetch_one(&mut **transaction)
            .await?
            .id;
            let player = TournamentPlayer {
                id: player_id,
                user_id: None,
                guest_id: Some(guest.get_id().to_owned()),
                name: guest.get_name().to_owned(),
                seeding: (index + 1).try_into().expect("seeding"),
            };
            players.push(player);
        }

        Ok(players)
    }

    /// Read all players for `tournament_id`
    pub async fn read_for_tournament(
        transaction: SqlxTransaction<'_, '_>,
        tournament_id: ID,
    ) -> Result<Vec<TournamentPlayer>, SqlxError> {
        let players = sqlx::query_as!(
            PlayerRecord,
            r#"
SELECT
    players.id,
    user_id,
    guest_id,
    COALESCE (U.name, G.name, '') as name,
    seeding_index
FROM players
LEFT JOIN users U ON players.user_id = U.id
LEFT JOIN guests G ON players.guest_id = G.id
WHERE tournament_id = $1            
            "#,
            tournament_id,
        )
        .fetch_all(&mut **transaction)
        .await?;
        Ok(players.into_iter().map(std::convert::Into::into).collect())
    }
}
