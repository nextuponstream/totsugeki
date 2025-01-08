//! Bracket repository

use crate::resources::PaginatedGenericResource;
use crate::tournaments::ID;
use crate::tournaments::{ParticipantError, Tournament};
use crate::tournaments::{ReportResultInput, TournamentRecord};
use crate::users::registration::UserRecord;
use serde::{Deserialize, Serialize};
use sqlx::error::Error as SqlxError;
use sqlx::types::Json as SqlxJson;
use sqlx::{Postgres, Transaction};
use thiserror::Error;
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::result::{MatchFormat, Score};
use totsugeki_core::matches::Match;
use totsugeki_core::player::Player;
use totsugeki_core::player::{Participants, PlayerID};
use totsugeki_core::validation::AutomaticMatchValidationMode;
use tracing::error;

/// Interact with brackets in postgres database using sqlx
#[derive(Debug)]
pub(crate) struct TournamentService {}

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

impl TournamentService {
    /// Create bracket and set creator `user_id` as tournament organiser
    pub async fn create(
        transaction: &mut Transaction<'_, Postgres>,
        tournament: &Tournament,
        double_elimination_bracket: &DoubleEliminationBracket,
        user_id: ID,
    ) -> Result<(), SqlxError> {
        let _ = sqlx::query!(
            "INSERT INTO tournaments (id, name, matches, participants) VALUES ($1, $2, $3, $4)",
            tournament.get_id().0,
            tournament.get_name(),
            SqlxJson(double_elimination_bracket.get_matches()) as _,
            SqlxJson(tournament.get_participants()) as _,
        )
        .execute(&mut **transaction)
        .await?;
        let _ = sqlx::query!(
            "INSERT INTO tournament_organisers (tournament_id, user_id) VALUES ($1, $2)",
            tournament.get_id().0,
            user_id,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    /// User joins bracket
    pub async fn join(
        transaction: &mut Transaction<'_, Postgres>,
        tournament_id: ID,
        user: UserRecord,
    ) -> Result<Option<(Tournament, DoubleEliminationBracket, bool)>, Error> {
        let Some(tournament_record) = sqlx::query_as!(
        TournamentRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>"  from tournaments WHERE id = $1"#,
        tournament_id,
        )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
            .fetch_optional(&mut **transaction).await? else {
            return Ok(None);
        };
        let is_tournament_organiser = sqlx::query!(
            r#"SELECT tournament_id, user_id from tournament_organisers WHERE user_id = $1 AND tournament_id = $2"#,
               user.id,
               tournament_id
        ).fetch_optional(&mut **transaction).await?.is_some();

        let (mut tournament, _) = tournament_record.parse();

        if let Err(e) =
            tournament.add_participant(Player::from((PlayerID::new(user.id), user.name)))
        {
            return match e {
                ParticipantError::AlreadyPresent => Err(Error::PlayerAlreadyPresent),
            };
        };

        let bracket = DoubleEliminationBracket::create(
            Seeding::new(
                tournament
                    .get_participants()
                    .0
                    .iter()
                    .map(Player::get_id)
                    .collect(),
            )
            .expect("should update seeding of bracket with tournament valid seeding"),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );

        Ok(Some((tournament, bracket, is_tournament_organiser)))
    }

    /// Returns bracket in database and boolean if user is a tournament organiser of that bracket
    pub async fn read_for_user(
        transaction: &mut Transaction<'_, Postgres>,
        tournament_id: ID,
        user_id: Option<ID>,
    ) -> Result<Option<(Tournament, DoubleEliminationBracket, bool)>, Error> {
        let Some(tournament_record) = sqlx::query_as!(
        TournamentRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>"  from tournaments WHERE id = $1"#,
        tournament_id,
        )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_optional(&mut **transaction).await? else {
            return Ok(None);
        };
        let is_tournament_organiser = match user_id {
            Some(to_id) => {
                sqlx::query!(
                    r#"SELECT tournament_id, user_id from tournament_organisers WHERE user_id = $1 AND tournament_id = $2"#,
                    to_id,
                    tournament_id
                ).fetch_optional(&mut **transaction).await?.is_some()
            }
            None => false,
        };

        let (tournament, bracket) = tournament_record.parse();

        Ok(Some((tournament, bracket, is_tournament_organiser)))
    }

    /// Update bracket with result
    pub async fn update_with_result(
        transaction: &mut Transaction<'_, Postgres>,
        tournament_id: ID,
        report: &ReportResultInput,
    ) -> Result<
        Option<(Tournament, DoubleEliminationBracket)>,
        crate::tournaments::update_with_result::Error,
    > {
        let Some(tournament_record) = sqlx::query_as!(
        TournamentRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>" from tournaments WHERE id = $1"#,
        tournament_id,
        )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
            .fetch_optional(&mut **transaction)
            .await?
            else {
                return Ok(None);
            };
        let (tournament, bracket) = tournament_record.parse();

        // FIXME actual error handling
        let (bracket, _, _) = bracket.tournament_organiser_reports_result_dangerous(
            report.p1_id,
            Score(report.score_p1, report.score_p2),
            report.p2_id,
        )?;
        let _r = sqlx::query!(
            r#"
        UPDATE tournaments 
            SET matches = $1
        WHERE id = $2
        "#,
            SqlxJson(bracket.get_matches()) as _,
            tournament.get_id().0,
        )
        .execute(&mut **transaction)
        .await?;
        Ok(Some((tournament, bracket)))
    }
    /// List all brackets belonging to `user_id`
    pub async fn list(
        transaction: &mut Transaction<'_, Postgres>,
        sort_order: String,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<PaginatedGenericResource>, SqlxError> {
        let brackets = sqlx::query_as!(
            PaginatedGenericResource,
            r#"SELECT id, name, created_at, count(*) OVER() AS total from tournaments 
         ORDER BY
           CASE WHEN $1 = 'ASC' THEN created_at END ASC,
           CASE WHEN $1 = 'DESC' THEN created_at END DESC
         LIMIT $2
         OFFSET $3
         "#,
            sort_order,
            limit,
            offset
        )
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_all(&mut **transaction)
        .await?;
        Ok(brackets)
    }
    /// List all brackets belonging to `user_id`
    pub async fn user_tournaments(
        transaction: &mut Transaction<'_, Postgres>,
        sort_order: String,
        limit: i64,
        offset: i64,
        user_id: ID,
    ) -> Result<Vec<PaginatedGenericResource>, SqlxError> {
        // paginated results with total count: https://stackoverflow.com/a/28888696
        // not optimal : each rows contains the total
        // not optimal : you have to extract total from first row if you want the
        // count to be separated from rows
        // weird: need Option<i64> for total otherwise does not compile
        // why keep : it might be nice for the consumer to access total rows in
        // the returned row. Also, it works for the current use case (return all
        // rows)
        // NOTE: ASC/DESC as param https://github.com/launchbadge/sqlx/issues/3020#issuecomment-1919930408
        let brackets = sqlx::query_as!(
            PaginatedGenericResource,
            r#"SELECT id, name, created_at, count(*) OVER() AS total from tournaments 
         WHERE id IN (SELECT tournament_id FROM tournament_organisers WHERE user_id = $4)
         ORDER BY
           CASE WHEN $1 = 'ASC' THEN created_at END ASC,
           CASE WHEN $1 = 'DESC' THEN created_at END DESC
         LIMIT $2
         OFFSET $3
         "#,
            sort_order,
            limit,
            offset,
            user_id,
        )
        // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_all(&mut **transaction)
        .await?;

        Ok(brackets)
    }
}
