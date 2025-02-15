//! Matches lifecycle

use crate::repositories::matches::MatchRepository;
use crate::repositories::reports::ReportRepository;
use crate::types::{SqlxError, SqlxTransaction};
use crate::ID;
use bigdecimal::ToPrimitive;
use totsugeki_core::matches::result::Score;
use totsugeki_core::matches::Match;

/// Create and manage matches
pub struct MatchService;

impl MatchService {
    /// Report score for a match
    ///
    /// # Errors
    /// Returns an error if communication with the database fails.
    /// # Panics
    /// When data validation is missing and last defense is triggered
    pub async fn report_score(
        transaction: SqlxTransaction<'_, '_>,
        match_id: &ID,
        score: &Score,
        tournament_organiser_id: Option<&ID>,
        player_id: Option<&ID>,
    ) -> Result<(), SqlxError> {
        assert!(tournament_organiser_id.is_some() ^ player_id.is_some());

        let reports = ReportRepository::get_all(transaction, match_id).await?;

        let already_reported = reports.iter().any(|r| {
            if let (Some(p_id), Some(player_id)) = (r.player_id, player_id) {
                p_id == *player_id
            } else if let (Some(to_id), Some(tournament_organiser_id)) =
                (r.tournament_organiser_id, tournament_organiser_id)
            {
                to_id == *tournament_organiser_id
            } else {
                unreachable!()
            }
        });
        if already_reported {
            todo!("report error")
        }

        ReportRepository::create(
            transaction,
            match_id,
            player_id,
            tournament_organiser_id,
            score,
        )
        .await?;
        Ok(())
    }

    /// Update many matches with corresponding scores
    ///
    /// # Errors
    /// Returns an error if communication with the database fails.
    /// # Panics
    /// Failed type coercion
    pub async fn update_many(
        transaction: SqlxTransaction<'_, '_>,
        matches: &[Match],
    ) -> Result<(), SqlxError> {
        for m in matches {
            let high_seed: i8 = m.get_seeds()[0]
                .try_into()
                .expect("type coercion for high seed");
            let low_seed: i8 = m.get_seeds()[1]
                .try_into()
                .expect("type coercion for low seed");
            // println!("{m}");
            // println!("---");
            // println!(
            //     "{}\n{:?}\n{:?}\n{:?}\n{:?}\n{}\n{}",
            //     m.get_id(),
            //     high_seed.to_i16(),
            //     m.get_players()[0].0,
            //     low_seed.to_i16(),
            //     m.get_players()[1].0,
            //     "first_to_n",
            //     2,
            // );
            sqlx::query!(
                r#"
UPDATE matches
SET
    high_seed = $2,
    high_seed_player = $3,
    low_seed = $4,
    low_seed_player = $5,
    format = $6,
    format_n = $7
WHERE id = $1
"#,
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

            let (high_seed_score, low_seed_score) = match m.get_score() {
                None => (None, None),
                Some(s) => (
                    Some(s.0.to_i16().expect("high seed score type coercion")),
                    Some(s.1.to_i16().expect("low seed score type coercion")),
                ),
            };

            // update score
            sqlx::query!(
                r#"
INSERT INTO match_final_scores (
    match_id,
    high_seed_player_score,
    low_seed_player_score,
    winner,
    automatic_loser
) VALUES ($1, $2, $3, $4, $5)
ON CONFLICT (match_id) DO UPDATE
    SET high_seed_player_score = excluded.high_seed_player_score,
        low_seed_player_score  = excluded.low_seed_player_score,
        winner                 = excluded.winner,
        automatic_loser        = excluded.automatic_loser
            "#,
                m.get_id(),
                high_seed_score,
                low_seed_score,
                m.get_winner().0,
                m.get_automatic_loser().0
            )
            .execute(&mut **transaction)
            .await?;
        }
        Ok(())
    }
    /// Delete all matches of given tournament
    ///
    /// # Errors
    /// Returns an error if communication with the database fails.
    pub async fn delete(
        transaction: SqlxTransaction<'_, '_>,
        tournament_id: &ID,
    ) -> Result<(), SqlxError> {
        sqlx::query!(
            r#"
DELETE FROM tournament_matches
WHERE tournament_id = $1;
        "#,
            tournament_id,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    /// Create matches for tournament
    ///
    /// # Errors
    /// Returns an error if communication with the database fails.
    /// # Panics
    /// Failed type coercion
    pub async fn create(
        transaction: SqlxTransaction<'_, '_>,
        tournament_id: &ID,
        matches: &[Match],
    ) -> Result<(), SqlxError> {
        MatchRepository::store_many(transaction, matches).await?;

        for (index, m) in matches.iter().enumerate() {
            sqlx::query!(
                r#"
INSERT INTO tournament_matches (
    tournament_id,
    match_id,        
    pos
) VALUES ($1, $2, $3)    
        "#,
                tournament_id,
                m.get_id(),
                (index + 1).to_i16().expect("type coercion of position"),
            )
            .execute(&mut **transaction)
            .await?;
        }

        Ok(())
    }
}
