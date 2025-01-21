//! Bracket repository

use crate::resources::PaginatedGenericResource;
use crate::tournaments::ID;
use crate::tournaments::{ParticipantError, Tournament};
use crate::tournaments::{ReportResultInput, TournamentRecord};
use crate::types::{SqlxError, SqlxTransaction};
use crate::users::registration::UserRecord;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::Match;
use tracing::error;

/// Interact with brackets in postgres database using sqlx
#[derive(Debug)]
#[deprecated(note = "use TournamentService instead")]
pub(crate) struct TournamentServiceOld {}

/// All errors when joining a bracket
#[derive(Error, Debug)]
pub(crate) enum Error {
    #[error("Unrecoverable database error")]
    /// Error with postgres, unrecoverable
    Sqlx(SqlxError),
    /// Inconsistent state in the client
    #[error("player tried to join bracket but they are already in")]
    PlayerAlreadyPresent,
}

impl From<SqlxError> for Error {
    fn from(err: SqlxError) -> Self {
        Self::Sqlx(err)
    }
}

/// Matches raw value
#[derive(Deserialize, Serialize)]
pub struct MatchesRaw(pub Vec<Match>);

impl TournamentServiceOld {
    /// Update bracket with result
    pub async fn update_with_result<'a>(
        transaction: SqlxTransaction<'a, '_>,
        tournament_id: ID,
        report: &ReportResultInput,
    ) -> Result<
        Option<(Tournament, DoubleEliminationBracket)>,
        crate::tournaments::update_with_result::Error,
    > {
        // let Some(tournament_record) = sqlx::query_as!(
        // TournamentRecord,
        // r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>" from tournaments WHERE id = $1"#,
        // tournament_id,
        // )
        //     // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        //     .fetch_optional(&mut **transaction)
        //     .await?
        //     else {
        //         return Ok(None);
        //     };
        // let (tournament, bracket) = tournament_record.parse();
        //
        // // FIXME actual error handling
        // let (bracket, _, _) = bracket.tournament_organiser_reports_result_dangerous(
        //     report.p1_id,
        //     Score(report.score_p1, report.score_p2),
        //     report.p2_id,
        // )?;
        // let _r = sqlx::query!(
        //     r#"
        // UPDATE tournaments
        //     SET matches = $1
        // WHERE id = $2
        // "#,
        //     SqlxJson(bracket.get_matches()) as _,
        //     tournament.get_id().0,
        // )
        // .execute(&mut **transaction)
        // .await?;
        // Ok(Some((tournament, bracket)))
        todo!()
    }
}
