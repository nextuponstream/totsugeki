//! bracket management

mod create;
mod join;
mod list;
mod new;
mod report_result;
mod save_bracket_from_steps;
mod show;
mod update_with_result;
mod user_brackets;

// Flatten exports when reusing
pub(crate) use crate::brackets::create::*;
pub(crate) use crate::brackets::join::*;
pub(crate) use crate::brackets::list::*;
pub(crate) use crate::brackets::new::*;
pub(crate) use crate::brackets::report_result::*;
pub(crate) use crate::brackets::save_bracket_from_steps::*;
pub(crate) use crate::brackets::show::*;
pub(crate) use crate::brackets::update_with_result::*;
pub(crate) use crate::brackets::user_brackets::*;

use crate::repositories::brackets::MatchesRaw;
use axum::{response::IntoResponse, Json as AxumJson};
use chrono::prelude::*;
use http::StatusCode;
use serde::{Deserialize, Serialize};
use sqlx::types::Json as SqlxJson;
use totsugeki::bracket::{
    double_elimination_variant::Variant as DoubleEliminationVariant, Bracket, Id,
};
use totsugeki::player::{Id as PlayerId, Participants, Player};
use totsugeki_display::loser_bracket::lines as loser_bracket_lines;
use totsugeki_display::loser_bracket::reorder as reorder_loser_bracket;
use totsugeki_display::winner_bracket::lines as winner_bracket_lines;
use totsugeki_display::winner_bracket::reorder as reorder_winner_bracket;
use totsugeki_display::{from_participants, BoxElement, MinimalMatch};
use validator::Validate;

/// List of players from which a bracket can be created
#[derive(Debug, Deserialize)]
pub struct ReportResultInput {
    /// current state of the bracket
    pub bracket: Bracket,
    /// First player
    pub p1_id: PlayerId,
    /// Second player
    pub p2_id: PlayerId,
    /// player 1 score
    pub score_p1: i8,
    /// player 2 score
    pub score_p2: i8,
}

/// Bracket to display. When there is less than 3 players, then there is nothing
/// to display
#[derive(Serialize, Debug, Deserialize)]
pub struct BracketDisplay {
    /// Winner bracket matches and lines to draw
    pub winner_bracket: Option<Vec<Vec<MinimalMatch>>>,
    /// Lines to draw between winner bracket matches
    pub winner_bracket_lines: Option<Vec<Vec<BoxElement>>>,
    /// Loser bracket matches and lines to draw
    pub loser_bracket: Option<Vec<Vec<MinimalMatch>>>,
    /// Lines to draw between loser bracket matches
    pub loser_bracket_lines: Option<Vec<Vec<BoxElement>>>,
    /// Grand finals
    pub grand_finals: Option<MinimalMatch>,
    /// Grand finals reset
    pub grand_finals_reset: Option<MinimalMatch>,
    /// Bracket object to update
    pub bracket: Bracket,
    /// true if user requesting the data is also a TO
    pub is_tournament_organiser: bool,
    /// true if user requesting the data participates
    pub is_participant: bool,
}

/// List of players from which a bracket can be created
#[derive(Deserialize, Serialize, Debug, Validate)]
pub struct CreateBracketForm {
    #[validate(length(min = 1))]
    /// bracket names
    pub bracket_name: String,
    /// player names
    pub player_names: Vec<String>,
}

/// Result reported by player
///
/// FIXME there probably is a less computive intensive way to save steps of
/// match, like only saving relevant match ID to update. But it's not there.
/// Then this will do.
#[derive(Deserialize, Serialize, Debug)]
pub struct PlayerMatchResultReport {
    /// high seed player
    pub p1_id: PlayerId,
    /// low seed player
    pub p2_id: PlayerId,
    /// score of player 1
    pub score_p1: i8,
    /// score of player 2
    pub score_p2: i8,
}

/// List of players from which a bracket can be created
#[derive(Deserialize, Serialize, Debug)]
pub struct BracketState {
    /// bracket names
    pub bracket_name: String,
    /// player names
    pub players: Vec<Player>,
    ///  results in order of replay
    pub results: Vec<PlayerMatchResultReport>,
}

/// Breaks down bracket in small parts to be presented by UI
fn breakdown(
    bracket: Bracket,
    user_id: Option<totsugeki::player::Id>,
    is_tournament_organiser: bool,
) -> impl IntoResponse {
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let winner_bracket_matches = dev.partition_winner_bracket();
    let winner_bracket_rounds = match winner_bracket_matches.clone() {
        Some(winner_bracket_matches) => {
            let mut winner_bracket_rounds = vec![];
            for r in winner_bracket_matches {
                let round = r
                    .iter()
                    .map(|m| from_participants(m, &bracket.get_participants()))
                    .collect();
                winner_bracket_rounds.push(round);
            }

            reorder_winner_bracket(&mut winner_bracket_rounds);
            Some(winner_bracket_rounds)
        }
        None => None,
    };
    let maybe_winner_bracket_lines = match winner_bracket_rounds.clone() {
        Some(winner_bracket_rounds) => winner_bracket_lines(&winner_bracket_rounds),
        None => None,
    };

    let lower_bracket_matches = dev.partition_loser_bracket();
    let loser_bracket_rounds = match lower_bracket_matches {
        Some(lower_bracket_matches) => {
            let mut loser_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
            for r in lower_bracket_matches {
                let round = r
                    .iter()
                    .map(|m| from_participants(m, &bracket.get_participants()))
                    .collect();
                loser_bracket_rounds.push(round);
            }
            reorder_loser_bracket(&mut loser_bracket_rounds);
            Some(loser_bracket_rounds)
        }
        None => None,
    };
    let maybe_loser_bracket_lines = match loser_bracket_rounds.clone() {
        Some(loser_bracket_rounds) => loser_bracket_lines(loser_bracket_rounds),
        None => None,
    };

    let (gf, gf_reset) = match dev.grand_finals_and_reset() {
        Some((gf, gf_reset)) => {
            let gf = from_participants(&gf, &bracket.get_participants());
            let gf_reset = from_participants(&gf_reset, &bracket.get_participants());
            (Some(gf), Some(gf_reset))
        }
        None => (None, None),
    };

    let is_participant = match user_id {
        Some(participant_id) => bracket.get_participants().get(participant_id).is_some(),
        None => false,
    };

    let bracket = BracketDisplay {
        winner_bracket: winner_bracket_rounds,
        winner_bracket_lines: maybe_winner_bracket_lines,
        loser_bracket: loser_bracket_rounds,
        loser_bracket_lines: maybe_loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
        is_participant,
        is_tournament_organiser,
    };
    tracing::info!("displaying bracket {}", bracket.bracket.get_id());
    tracing::debug!("displaying bracket {:?}", bracket);
    (StatusCode::OK, AxumJson(bracket)).into_response()
}

#[derive(Serialize, Deserialize)]
/// 201 response
pub struct GenericResourceCreated {
    /// Resource ID
    pub id: Id,
}

/// Bracket in database
#[derive(Serialize, Deserialize, sqlx::FromRow)]
pub(crate) struct BracketRecord {
    /// bracket ID
    pub id: Id,
    /// name
    pub name: String,
    /// creation date
    pub created_at: DateTime<Utc>,
    /// matches
    pub matches: SqlxJson<MatchesRaw>,
    /// participants
    pub participants: SqlxJson<Participants>,
}
