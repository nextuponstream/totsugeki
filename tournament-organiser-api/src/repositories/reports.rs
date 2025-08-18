//! Match report persistence

use crate::types::{SqlxError, SqlxTransaction};
use crate::ID;
use bigdecimal::ToPrimitive;
use totsugeki_core::matches::result::Score;

/// Persist reports for tournament matches
pub(crate) struct ReportRepository {}

/// Report from database
#[allow(dead_code)]
pub struct Report {
    /// ID of report
    pub(crate) id: ID,
    /// ID of match
    pub(crate) match_id: ID,
    /// ID of player
    pub(crate) player_id: Option<ID>,
    /// ID of tournament organiser
    pub(crate) tournament_organiser_id: Option<ID>,
    /// (left) score
    pub(crate) high_seed_player_score: usize,
    /// (right) score
    pub(crate) low_seed_player_score: usize,
}

/// Report from database
pub struct ReportRecord {
    /// ID of report
    pub(crate) id: ID,
    /// ID of match
    pub(crate) match_id: ID,
    /// ID of player
    pub(crate) player_id: Option<ID>,
    /// ID of tournament organiser
    pub(crate) tournament_organiser_id: Option<ID>,
    /// (left) score
    pub(crate) high_seed_player_score: i16,
    /// (right) score
    pub(crate) low_seed_player_score: i16,
}

impl From<ReportRecord> for Report {
    fn from(value: ReportRecord) -> Self {
        Self {
            id: value.id,
            match_id: value.match_id,
            player_id: value.player_id,
            tournament_organiser_id: value.tournament_organiser_id,
            high_seed_player_score: value
                .high_seed_player_score
                .to_usize()
                .expect("high seed type coercion"),
            low_seed_player_score: value
                .low_seed_player_score
                .to_usize()
                .expect("low seed type coercion"),
        }
    }
}

impl ReportRepository {
    /// Get all reports for `match_id`
    pub async fn get_all(
        transaction: SqlxTransaction<'_, '_>,
        match_id: &ID,
    ) -> Result<Vec<Report>, SqlxError> {
        let reports = sqlx::query_as!(
            ReportRecord,
            r#"
SELECT
    id,
    match_id,
    player_id,
    tournament_organiser_id,
    high_seed_player_score,
    low_seed_player_score
FROM reports
WHERE match_id = $1
"#,
            match_id,
        )
        .fetch_all(&mut **transaction)
        .await?;
        Ok(reports.into_iter().map(std::convert::Into::into).collect())
    }

    /// Create report
    pub async fn create(
        transaction: SqlxTransaction<'_, '_>,
        match_id: &ID,
        player_id: Option<&ID>,
        tournament_organiser_id: Option<&ID>,
        score: &Score,
    ) -> Result<(), SqlxError> {
        assert!(tournament_organiser_id.is_some() ^ player_id.is_some());
        sqlx::query!(
            r#"
INSERT INTO reports (
    match_id,
    player_id,
    tournament_organiser_id,
    high_seed_player_score,
    low_seed_player_score
) VALUES ($1, $2, $3, $4, $5)
        "#,
            match_id,
            player_id,
            tournament_organiser_id,
            score.0.to_i16().expect("high seed score type coercion"),
            score.1.to_i16().expect("low seed score type coercion")
        )
        .execute(&mut **transaction)
        .await?;
        Ok(())
    }
}
