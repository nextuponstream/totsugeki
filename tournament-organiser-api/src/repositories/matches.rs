//! Store tournament matches

use crate::types::{SqlxError, SqlxTransaction};
use bigdecimal::ToPrimitive;
use totsugeki_core::matches::result::MatchFormat;
use totsugeki_core::matches::Match;
use totsugeki_core::opponent::Opponent;
use totsugeki_core::ID;

/// Persist matches
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
            println!("{m}");
            println!("---");
            println!(
                "{}\n{:?}\n{:?}\n{:?}\n{:?}\n{}\n{}",
                m.get_id(),
                high_seed.to_i16(),
                m.get_players()[0].0,
                low_seed.to_i16(),
                m.get_players()[1].0,
                "first_to_n",
                2,
            );
            sqlx::query!(
                r#"
INSERT into matches (
id,
high_seed,
high_seed_player,
low_seed,
low_seed_player,
format,
format_n
) VALUES ($1, $2, $3, $4, $5, $6, $7)"#,
                m.get_id(),
                high_seed.to_i16(),
                m.get_players()[0].0,
                low_seed.to_i16(),
                m.get_players()[1].0,
                "first_to_n",
                2.into(),
            )
            .execute(&mut **transaction)
            .await?;
        }
        Ok(())
    }
}

/// Tournament match
pub(crate) struct TournamentMatch {
    /// relationship tournament-match ID
    pub id: ID,
    /// tournament ID
    pub tournament_id: ID,
    /// match ID
    pub match_id: ID,
    /// order of match (unique for each match in tournament)
    pub pos: i16,
    /// presumed stronger player
    pub high_seed_player: Option<ID>,
    /// presumed weakest player
    pub low_seed_player: Option<ID>,
    /// left seed (highest seed) for the presumed strongest predicted player
    pub high_seed: i16,
    /// right seed (lowest seed) for the presumed weakest predicted player
    pub low_seed: i16,
    /// match format (example: first to X)
    pub format: String,
    /// additionnal information about match format (example: first to 3)
    pub format_n: i16,
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
