//! New unsaved bracket

use crate::brackets::{breakdown, CreateBracketForm};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use totsugeki::bracket::Bracket;
use tracing::instrument;

/// Return a newly instantiated bracket from ordered (=seeded) player names for
/// display purposes
///
/// # Panics
/// When bracket cannot be converted to double elimination bracket
///
/// # Errors
/// May return 500 error when bracket cannot be parsed
#[instrument(name = "new_bracket")]
pub async fn new_bracket(Json(form): Json<CreateBracketForm>) -> impl IntoResponse {
    tracing::debug!("new bracket");

    let mut bracket = Bracket::default();
    bracket = bracket.update_name(form.bracket_name);
    for name in form.player_names {
        // FIXME into
        let Ok(tmp) = bracket.add_participant(name.as_str()) else {
            // FIXME actual error handling
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp.0;
    }

    Ok(breakdown(bracket, None, false))
}
