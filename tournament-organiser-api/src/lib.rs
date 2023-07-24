//! Bracket management and visualiser library for admin dashboard
use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use totsugeki::bracket::Bracket;
use totsugeki::player::Participants;
use totsugeki::{
    bracket::double_elimination_variant::Variant as DoubleEliminationVariant,
    matches::Id as MatchId, matches::Match, player::Id as PlayerId,
};

/// Bracket to display
#[derive(Serialize, Debug)]
struct BracketDisplay {
    /// Winner bracket matches and lines to draw
    winner_bracket: Vec<bool>,
    /// Loser bracket matches and lines to draw
    loser_bracket: Vec<bool>,
    /// Grand finals
    grand_finals: MinimalMatch,
    /// Grand finals reset
    grand_finals_reset: MinimalMatch,
}

/// List of players from which a bracket can be created
#[derive(Deserialize)]
pub struct PlayerList {
    /// player names
    names: Vec<String>,
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
pub async fn new_bracket_from_players(Json(player_list): Json<PlayerList>) -> impl IntoResponse {
    tracing::debug!("new bracket from players: {:?}", player_list.names);
    let mut bracket = Bracket::default();
    for name in player_list.names {
        let Ok(tmp) = bracket.add_participant(name.as_str()) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp;
    }
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    let Ok((gf, gf_reset)) = dev.grand_finals_and_reset() else {
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let participants = bracket.get_participants();
    let gf = from_participants(&gf, &participants);
    let gf_reset = from_participants(&gf_reset, &participants);

    let bracket = BracketDisplay {
        winner_bracket: vec![],
        loser_bracket: vec![],
        grand_finals: gf,
        grand_finals_reset: gf_reset,
    };
    tracing::debug!("created bracket {:?}", bracket);
    Ok(Json(bracket))
}

#[derive(Debug, Clone, Serialize)]
/// Strict necessary information to use when displaying a match in UI
pub struct MinimalMatch {
    /// Match identifier
    id: MatchId,
    /// Names of players participating in match
    players: [String; 2],
    /// Score of match
    score: (i8, i8),
    /// Expected seeds of player in match
    seeds: [usize; 2],
    /// Indicate which row it belongs to, starting from 0 index
    row_hint: Option<usize>,
}

/// Convert match struct from Totsugeki library into minimal struct, using
/// `participants` to fill in name of players.
///
///
fn from_participants(m: &Match, participants: &Participants) -> MinimalMatch {
    let list = participants.get_players_list();
    let players: Vec<(PlayerId, String)> =
        list.iter().map(|p| (p.get_id(), p.get_name())).collect();
    let player1 = m.get_players()[0].get_name(&players);
    let player2 = m.get_players()[1].get_name(&players);
    MinimalMatch {
        id: m.get_id(),
        players: [player1, player2],
        score: m.get_score(),
        seeds: m.get_seeds(),
        row_hint: None,
    }
}
