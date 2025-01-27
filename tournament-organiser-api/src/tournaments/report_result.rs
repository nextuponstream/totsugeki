//! Report result without saving to database

use crate::http::ErrorSlug;
use crate::tournaments::{breakdown, ReportResultInput};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use totsugeki_core::matches::result::Score;
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
#[instrument(name = "score", skip(report))]
pub async fn score(Json(report): Json<ReportResultInput>) -> impl IntoResponse {
    todo!()
    // tracing::debug!("new reported result");
    // let bracket = report.bracket;
    // let tournament = report.tournament;
    //
    // let Ok((bracket, _, _)) = bracket.tournament_organiser_reports_result_dangerous(
    //     report.player1_id,
    //     Score(report.score_p1, report.score_p2),
    //     report.player2_id,
    // ) else {
    //     // FIXME actual error handling
    //     return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
    // };
    // // People allowed to report are tournament organiser
    // Ok((StatusCode::OK, breakdown(&tournament, bracket, None, true)))
}
