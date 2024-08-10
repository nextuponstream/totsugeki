//! Update bracket with result

use crate::brackets::{breakdown, ReportResultInput};
use crate::repositories::brackets::BracketRepository;
use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use sqlx::PgPool;
use totsugeki::bracket::Id;
use tracing::instrument;

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
    Path(bracket_id): Path<Id>,
    Json(report): Json<ReportResultInput>,
) -> impl IntoResponse {
    // FIXME check if user can edit bracket using tournament_organisers table
    tracing::debug!("new reported result");
    let mut transaction = pool.begin().await.unwrap();
    let bracket =
        match BracketRepository::update_with_result(&mut transaction, bracket_id, &report).await {
            Ok(Some(bracket)) => bracket,
            Ok(None) => return Err(StatusCode::NOT_FOUND),
            Err(e) => {
                tracing::warn!("Cannot update bracket {bracket_id} with result {report:?}: {e:?}");
                return Err(StatusCode::INTERNAL_SERVER_ERROR);
            }
        };
    transaction.commit().await.unwrap();
    Ok(breakdown(bracket, None, true))
}
