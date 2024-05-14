//! Bracket repository

use serde::{Deserialize, Serialize};
use sqlx::error::Error as SqlxError;
use sqlx::types::Json as SqlxJson;
use sqlx::PgPool;

use totsugeki::bracket::Bracket;
use totsugeki::bracket::Error as TotsugekiError;
use totsugeki::matches::Match;
use totsugeki::player::Id;
use totsugeki::player::Participants;

use crate::brackets::{BracketRecord, ReportResultInput};

/// Interact with brackets in postgres database using sqlx
#[derive(Debug)]
pub(crate) struct BracketRepository {
    /// Connection pool to database
    pool: PgPool,
}

/// All errors from using sqlx
#[derive(Debug)]
pub(crate) enum Error {
    // reason for allow lint: might be needed to track down bug later
    #[allow(dead_code)]
    /// Error with sqlx, unrecoverable
    Sqlx(SqlxError),
    /// Expected error when using library
    #[allow(dead_code)]
    Bracket(TotsugekiError),
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
    /// Create new Bracket repository and interface with postgres database
    pub fn new(pool: PgPool) -> BracketRepository {
        Self { pool }
    }

    /// Create bracket and set creator `user_id` as tournament organiser
    pub async fn create(self, bracket: &Bracket, user_id: Id) -> Result<(), Error> {
        let mut transaction = self.pool.begin().await?;
        let _ = sqlx::query!(
            "INSERT INTO brackets (id, name, matches, participants) VALUES ($1, $2, $3, $4)",
            bracket.get_id(),
            bracket.get_name(),
            SqlxJson(bracket.get_matches()) as _,
            SqlxJson(bracket.get_participants()) as _,
        )
        .execute(&mut *transaction)
        .await?;
        let _ = sqlx::query!(
            "INSERT INTO tournament_organisers (bracket_id, user_id) VALUES ($1, $2)",
            bracket.get_id(),
            user_id,
        )
        .execute(&mut *transaction)
        .await?;

        transaction.commit().await?;

        Ok(())
    }
    /// Read bracket in database
    pub async fn read(self, bracket_id: Id) -> Result<Option<Bracket>, Error> {
        let Some(b) = sqlx::query_as!(
        BracketRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>"  from brackets WHERE id = $1"#,
        bracket_id,
    )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
            .fetch_optional(&self.pool)
            .await
            .expect("fetch result") else {
            return Ok(None)
        };
        let bracket = Bracket::assemble(b.id, b.name, b.participants.0, b.matches.0 .0);

        Ok(Some(bracket))
    }

    /// Update bracket with result
    pub async fn update_with_result(
        self,
        bracket_id: Id,
        report: &ReportResultInput,
    ) -> Result<Option<Bracket>, Error> {
        let mut transaction = self.pool.begin().await?;
        let Some(b) = sqlx::query_as!(
        BracketRecord,
        r#"SELECT id, name, matches as "matches: SqlxJson<MatchesRaw>", created_at, participants as "participants: SqlxJson<Participants>" from brackets WHERE id = $1"#,
        bracket_id,
    )
            // https://github.com/tokio-rs/axum/blob/1e5be5bb693f825ece664518f3aa6794f03bfec6/examples/sqlx-postgres/src/main.rs#L71
            .fetch_optional(&mut *transaction)
            .await
            .expect("fetch result") else {
            return Ok(None);
        };
        let bracket = Bracket::assemble(b.id, b.name, b.participants.0, b.matches.0 .0);

        let bracket = match bracket.tournament_organiser_reports_result(
            report.p1_id,
            (report.score_p1, report.score_p2),
            report.p2_id,
        ) {
            Ok((bracket, _, _)) => bracket,
            Err(e) => {
                // TODO from implementation for totsugeki error
                tracing::warn!("{e:?}");
                return Err(Error::Bracket(e));
            }
        };

        let _r = sqlx::query!(
            r#"
        UPDATE brackets
            SET matches = $1
        WHERE id = $2
        "#,
            SqlxJson(bracket.get_matches()) as _,
            bracket.get_id(),
        )
        .execute(&mut *transaction)
        .await
        .expect("new bracket replayed from steps");
        transaction.commit().await?;
        Ok(Some(bracket))
    }
}
