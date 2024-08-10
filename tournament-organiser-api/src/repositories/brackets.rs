//! Bracket repository

use serde::{Deserialize, Serialize};
use sqlx::error::Error as SqlxError;
use sqlx::types::Json as SqlxJson;
use sqlx::{Postgres, Transaction};

use totsugeki::bracket::Bracket;
use totsugeki::matches::Match;
use totsugeki::player::Participants;
use totsugeki::player::{Id, Player};

use crate::brackets::{BracketRecord, ReportResultInput};
use crate::resources::PaginatedGenericResource;
use crate::users::registration::UserRecord;
use thiserror::Error;
use totsugeki::bracket::participants::Error as JoinError;
use totsugeki::player::Error as PlayerError;
use tracing::error;

/// Interact with brackets in postgres database using sqlx
#[derive(Debug)]
pub(crate) struct BracketRepository {}

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

impl BracketRepository {
    /// Create bracket and set creator `user_id` as tournament organiser
    pub async fn create(
        transaction: &mut Transaction<'_, Postgres>,
        bracket: &Bracket,
        user_id: Id,
    ) -> Result<(), SqlxError> {
        let _ = sqlx::query!(
            "INSERT INTO brackets (id, name, matches, participants) VALUES ($1, $2, $3, $4)",
            bracket.get_id(),
            bracket.get_name(),
            SqlxJson(bracket.get_matches()) as _,
            SqlxJson(bracket.get_participants()) as _,
        )
        .execute(&mut **transaction)
        .await?;
        let _ = sqlx::query!(
            "INSERT INTO tournament_organisers (bracket_id, user_id) VALUES ($1, $2)",
            bracket.get_id(),
            user_id,
        )
        .execute(&mut **transaction)
        .await?;

        Ok(())
    }

    /// User joins bracket
    pub async fn join(
        transaction: &mut Transaction<'_, Postgres>,
        bracket_id: Id,
        user: UserRecord,
    ) -> Result<Option<(Bracket, bool)>, Error> {
        let Some(b) = sqlx::query_as!(
        BracketRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>"  from brackets WHERE id = $1"#,
        bracket_id,
        )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
            .fetch_optional(&mut **transaction).await? else {
            return Ok(None);
        };
        let is_tournament_organiser = sqlx::query!(
            r#"SELECT bracket_id, user_id from tournament_organisers WHERE user_id = $1 AND bracket_id = $2"#,
               user.id,
               bracket_id
        ).fetch_optional(&mut **transaction).await?.is_some();

        let bracket = Bracket::assemble(b.id, b.name, b.participants.0, b.matches.0 .0);

        let bracket = match bracket.join(Player::from((user.id, user.name))) {
            Ok(b) => b,
            Err(JoinError::ParticipantError(PlayerError::AlreadyPresent)) => {
                return Err(Error::PlayerAlreadyPresent)
            }
            // TODO handle edge case
            // Err(JoinError::BarredFromEntering(_, _)) => {
            //     panic!()
            // }
            Err(e) => {
                error!("{e:?}");
                panic!("{e:?}")
            }
        };

        Ok(Some((bracket, is_tournament_organiser)))
    }

    /// Returns bracket in database and boolean if user is a tournament organiser of that bracket
    pub async fn read_for_user(
        transaction: &mut Transaction<'_, Postgres>,
        bracket_id: Id,
        user_id: Option<totsugeki::player::Id>,
    ) -> Result<Option<(Bracket, bool)>, Error> {
        let Some(b) = sqlx::query_as!(
        BracketRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>"  from brackets WHERE id = $1"#,
        bracket_id,
        )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
        .fetch_optional(&mut **transaction).await? else {
            return Ok(None);
        };
        let is_tournament_organiser = match user_id {
            Some(to_id) => {
                sqlx::query!(
                    r#"SELECT bracket_id, user_id from tournament_organisers WHERE user_id = $1 AND bracket_id = $2"#,
                    to_id,
                    bracket_id
                ).fetch_optional(&mut **transaction).await?.is_some()
            }
            None => false,
        };

        let bracket = Bracket::assemble(b.id, b.name, b.participants.0, b.matches.0 .0);

        Ok(Some((bracket, is_tournament_organiser)))
    }

    /// Update bracket with result
    pub async fn update_with_result(
        transaction: &mut Transaction<'_, Postgres>,
        bracket_id: Id,
        report: &ReportResultInput,
    ) -> Result<Option<Bracket>, SqlxError> {
        let Some(b) = sqlx::query_as!(
        BracketRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>" from brackets WHERE id = $1"#,
        bracket_id,
        )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
            .fetch_optional(&mut **transaction)
            .await?
            else {
                return Ok(None);
            };
        let bracket = Bracket::assemble(b.id, b.name, b.participants.0, b.matches.0 .0);

        // FIXME actual error handling
        let (bracket, _, _) = bracket
            .tournament_organiser_reports_result(
                report.p1_id,
                (report.score_p1, report.score_p2),
                report.p2_id,
            )
            .unwrap();
        let _r = sqlx::query!(
            r#"
        UPDATE brackets
            SET matches = $1
        WHERE id = $2
        "#,
            SqlxJson(bracket.get_matches()) as _,
            bracket.get_id(),
        )
        .execute(&mut **transaction)
        .await?;
        Ok(Some(bracket))
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
            r#"SELECT id, name, created_at, count(*) OVER() AS total from brackets
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
    pub async fn user_brackets(
        transaction: &mut Transaction<'_, Postgres>,
        sort_order: String,
        limit: i64,
        offset: i64,
        user_id: Id,
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
            r#"SELECT id, name, created_at, count(*) OVER() AS total from brackets
         WHERE id IN (SELECT bracket_id FROM tournament_organisers WHERE user_id = $4)
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
