//! New unsaved bracket

use crate::tournaments::Tournament;
use crate::tournaments::{breakdown, CreateBracketForm};
use axum::response::IntoResponse;
use axum::Json;
use http::StatusCode;
use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::result::MatchFormat;
use totsugeki_core::player::Player;
use totsugeki_core::validation::AutomaticMatchValidationMode;
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

    let mut tournament = Tournament::default();
    tournament.set_name(form.bracket_name);
    for name in form.player_names {
        let Ok(()) = tournament.add_participant(Player::new(name)) else {
            // FIXME actual error handling
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(tournament.get_participants().get_seeding())
            .expect("should use seeding from tournament organiser input"),
        AutomaticMatchValidationMode::Flexible, // FIXME from form
        MatchFormat::ft2(),                     // FIXME parse from form
        None,                                   // FIXME parse from form
    );

    Ok(breakdown(&tournament, bracket, None, false))
}
