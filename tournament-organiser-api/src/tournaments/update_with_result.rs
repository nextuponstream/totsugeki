//! Update bracket with result

use crate::http::{internal_error, ErrorSlug};
use crate::repositories::brackets::TournamentService;
use crate::tournaments::{breakdown, ReportResultInput};
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use sqlx::Error as SqlxError;
use sqlx::PgPool;
use thiserror::Error;
use totsugeki_core::bracket::Id;
use tracing::instrument;

/// Cannot update double elimination bracket with result
#[derive(Error, Debug)]
pub(crate) enum Error {
    /// Potentially recoverable
    #[error("{0}")]
    App(
        #[from]
        totsugeki_core::double_elimination_bracket::progression::DoubleEliminationReportResultError,
    ),
    /// Unrecoverable
    #[error("{0}")]
    SqlxError(#[from] SqlxError),
}

/// Returns updated bracket with result. Because there is no persistence, it's
/// obviously limited in that TO can manipulate localStorage to change the
/// bracket, but we are not worried about that right now. For now, the goal is
/// that it just works for normal use cases
///
/// # Panics
/// May panic if I fucked up
///
/// # Errors
/// Error 500 if a user gets out of sync with the bracket in the database and
/// the one displayed in the web page
// TODO report should be at debug level
#[instrument(name = "update_with_result", skip(report, pool))]
pub async fn update_with_result(
    State(pool): State<PgPool>,
    Path(tournament_id): Path<Id>,
    Json(report): Json<ReportResultInput>,
) -> impl IntoResponse {
    // FIXME check if user can edit bracket using tournament_organisers table
    tracing::debug!("new reported result");
    let mut transaction = pool.begin().await.map_err(internal_error)?;
    let (tournament, bracket) =
        match TournamentService::update_with_result(&mut transaction, tournament_id, &report).await
        {
            Ok(Some(bracket)) => bracket,
            Ok(None) => return Err(ErrorSlug::from(StatusCode::NOT_FOUND)),
            Err(Error::SqlxError(e)) => {
                tracing::error!(
                    "Cannot update tournament {tournament_id} with result {report:?}: {e:?}"
                );
                return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
            }
            Err(Error::App(e)) => {
                tracing::warn!(
                "Cannot use reported result to update tournament {}. Is frontend up to date?: {}",
                tournament_id,
                e
            );
                return Err(ErrorSlug::new(StatusCode::BAD_REQUEST, e.to_string()));
            }
        };
    transaction.commit().await.map_err(internal_error)?;
    Ok(breakdown(&tournament, bracket, None, true))
}
