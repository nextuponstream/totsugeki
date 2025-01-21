//! Store tournament matches

use crate::tournaments::TournamentID;
use crate::types::{SqlxError, SqlxTransaction};
use bigdecimal::ToPrimitive;
use sqlx::types::BigDecimal;
use totsugeki_core::matches::result::MatchFormat;
use totsugeki_core::matches::{Match, MatchID};
use totsugeki_core::opponent::Opponent;
use totsugeki_core::player::PlayerID;
use totsugeki_core::ID;

#[derive(Debug)]
pub(crate) struct MatchRepository {}

impl MatchRepository {
    /// Save matches to database
    pub async fn store_many<'a>(
        transaction: SqlxTransaction<'a, '_>,
        matches: Vec<Match>,
    ) -> Result<(), SqlxError> {
        for m in matches.iter() {
            let high_seed: i8 = m.get_seeds()[0].try_into().unwrap();
            let low_seed: i8 = m.get_seeds()[1].try_into().unwrap();
            sqlx::query!(
                r#"INSERT into matches (
            high_seed,
            high_seed_player,
            low_seed,
            low_seed_player,
            format,
            format_n
            ) VALUES ($1, $2, $3, $4, $5, $6)"#,
                high_seed.to_i16(),
                m.get_players()[0].0.map(|id| id.0),
                low_seed.to_i16(),
                m.get_players()[1].0.map(|id| id.0),
                "first_to_n",
                2.into(),
            )
            .execute(&mut **transaction)
            .await?;
        }
        Ok(())
    }

    pub async fn get_for_tournament<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: ID,
    ) -> Result<Vec<Match>, SqlxError> {
        let matches = sqlx::query_as!(
            TournamentMatch,
            r#"SELECT * from tournament_matches
            LEFT JOIN matches on tournament_matches.match_id = matches.id
            WHERE tournament_id = $1"#,
            tournament_id
        )
        .fetch_all(&mut **transaction)
        .await?;
        Ok(matches.into_iter().map(|v| v.into()).collect())
    }
}

struct TournamentMatch {
    id: ID,
    tournament_id: ID,
    match_id: MatchID,
    pos: BigDecimal,
    high_seed_player: Option<ID>,
    low_seed_player: Option<ID>,
    high_seed: BigDecimal,
    low_seed: BigDecimal,
    format: String,
    format_n: BigDecimal,
}

impl From<TournamentMatch> for Match {
    fn from(value: TournamentMatch) -> Self {
        let seeds: [usize; 2] = [
            value.high_seed.to_usize().expect("high seed"),
            value.low_seed.to_usize().expect("low seed"),
        ];
        let players: [Opponent; 2] = [value.high_seed_player.into(), value.low_seed_player.into()];
        let format: MatchFormat =
            MatchFormat::new(value.format_n.to_u8().expect("first to n")).unwrap();
        Match::new(Some(value.match_id), players, seeds, format).expect("match")
    }
}
