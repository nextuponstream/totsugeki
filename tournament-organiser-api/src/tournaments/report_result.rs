//! Report result without saving to database

use crate::http::ErrorSlug;
use crate::tournaments::{breakdown, GuestReportResultInput, Tournament};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use totsugeki_core::matches::result::Score;
use tracing::instrument;

/// Returns updated tournament with result. Because there is no persistence, it's
/// obviously limited in that TO can manipulate localStorage to change the
/// bracket, but we are not worried about that right now. For now, the goal is
/// that it just works for normal use cases
///
/// # Errors
/// If provided bracket in report is invalid
#[instrument(name = "report_result", skip(report))]
pub async fn report_result(Json(report): Json<GuestReportResultInput>) -> impl IntoResponse {
    tracing::debug!("new reported result");
    let bracket = report.bracket;
    let tournament: Tournament = report.tournament;

    let Ok((bracket, _, _)) = bracket.tournament_organiser_reports_result_dangerous(
        &report.player1_id,
        Score(report.score_p1, report.score_p2),
        &report.player2_id,
    ) else {
        // FIXME actual error handling
        return Err(ErrorSlug::from(StatusCode::INTERNAL_SERVER_ERROR));
    };
    // People allowed to report are tournament organiser
    Ok((StatusCode::OK, breakdown(&tournament, bracket, None, true)))
}
