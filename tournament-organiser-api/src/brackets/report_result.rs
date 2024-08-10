//! Report result without saving to database

use crate::brackets::{breakdown, ReportResultInput};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
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
#[instrument(name = "report_result", skip(report))]
pub async fn report_result(Json(report): Json<ReportResultInput>) -> impl IntoResponse {
    tracing::debug!("new reported result");
    let mut bracket = report.bracket;

    bracket = match bracket.tournament_organiser_reports_result(
        report.p1_id,
        (report.score_p1, report.score_p2),
        report.p2_id,
    ) {
        Ok((bracket, _, _)) => bracket,
        Err(e) => {
            // TODO deal with corner case where UI shows a bracket that is out
            //  of sync with database state and returns something to user
            tracing::warn!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    // People allowed to report are tournament organiser
    Ok(breakdown(bracket, None, true))
}
