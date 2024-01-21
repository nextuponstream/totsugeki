//! bracket management

use axum::{response::IntoResponse, Json};
use http::StatusCode;
use serde::{Deserialize, Serialize};
use totsugeki::bracket::{
    double_elimination_variant::Variant as DoubleEliminationVariant, Bracket,
};
use totsugeki::player::Id as PlayerId;
use totsugeki_display::loser_bracket::lines as loser_bracket_lines;
use totsugeki_display::loser_bracket::reorder as reorder_loser_bracket;
use totsugeki_display::winner_bracket::lines as winner_bracket_lines;
use totsugeki_display::winner_bracket::reorder as reorder_winner_bracket;
use totsugeki_display::{from_participants, BoxElement, MinimalMatch};

/// List of players from which a bracket can be created
#[derive(Deserialize)]
pub struct ReportResultInput {
    /// current state of the bracket
    bracket: Bracket,
    /// First player
    p1_id: PlayerId,
    /// Second player
    p2_id: PlayerId,
    /// player 1 score
    score_p1: i8,
    /// player 2 score
    score_p2: i8,
}

/// Bracket to display
#[derive(Serialize, Debug)]
struct BracketDisplay {
    /// Winner bracket matches and lines to draw
    winner_bracket: Vec<Vec<MinimalMatch>>,
    /// Lines to draw between winner bracket matches
    winner_bracket_lines: Vec<Vec<BoxElement>>,
    /// Loser bracket matches and lines to draw
    loser_bracket: Vec<Vec<MinimalMatch>>,
    /// Lines to draw between loser bracket matches
    loser_bracket_lines: Vec<Vec<BoxElement>>,
    /// Grand finals
    grand_finals: MinimalMatch,
    /// Grand finals reset
    grand_finals_reset: MinimalMatch,
    /// Bracket object to update
    bracket: Bracket,
}

/// List of players from which a bracket can be created
#[derive(Deserialize)]
pub struct PlayerList {
    /// player names
    names: Vec<String>,
}

/// Return a newly instanciated bracket from ordered (=seeded) player names
///
/// # Panics
/// When bracket cannot be converted to double elimination bracket
///
/// # Errors
/// May return 500 error when bracket cannot be parsed
pub async fn new_bracket_from_players(Json(player_list): Json<PlayerList>) -> impl IntoResponse {
    tracing::debug!("new bracket from players: {:?}", player_list.names);
    let mut bracket = Bracket::default();
    for name in player_list.names {
        let Ok(tmp) = bracket.add_participant(name.as_str()) else {
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        };
        bracket = tmp;
    }
    let participants = bracket.get_participants();
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let winner_bracket_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut winner_bracket_rounds = vec![];
    for r in winner_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        winner_bracket_rounds.push(round);
    }

    reorder_winner_bracket(&mut winner_bracket_rounds);
    let Some(winner_bracket_lines) = winner_bracket_lines(&winner_bracket_rounds) else {
        tracing::error!("winner bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(lower_bracket_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut loser_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lower_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        loser_bracket_rounds.push(round);
    }
    reorder_loser_bracket(&mut loser_bracket_rounds);
    let Some(loser_bracket_lines) = loser_bracket_lines(loser_bracket_rounds.clone()) else {
        tracing::error!("loser bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let (gf, gf_reset) = match dev.grand_finals_and_reset() {
        Ok((gf, bracket_reset)) => (gf, bracket_reset),
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let gf = from_participants(&gf, &participants);
    let gf_reset = from_participants(&gf_reset, &participants);

    let bracket = BracketDisplay {
        winner_bracket: winner_bracket_rounds,
        winner_bracket_lines,
        loser_bracket: loser_bracket_rounds,
        loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
    };
    tracing::debug!("updated bracket {:?}", bracket);
    Ok(Json(bracket))
}

/// Returns updated bracket with result. Because there is no persistence, it's
/// obviously limited in that TO can manipulate localStorage to change the
/// bracket but we are not worried about that right now. For now, the goal is
/// that it just works for normal use cases
///
/// # Panics
/// May panic if I fucked up
///
/// # Errors
/// Error 500 if a user gets out of sync with the bracket in the database and
/// the one displayed in the web page
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
            // of sync with database state and returns something to user
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };

    let participants = bracket.get_participants();
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let winner_bracket_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut winner_bracket_rounds = vec![];
    for r in winner_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        winner_bracket_rounds.push(round);
    }

    reorder_winner_bracket(&mut winner_bracket_rounds);
    let Some(winner_bracket_lines) = winner_bracket_lines(&winner_bracket_rounds) else {
        tracing::error!("winner bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let Ok(lower_bracket_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut lower_bracket_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lower_bracket_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        lower_bracket_rounds.push(round);
    }
    reorder_loser_bracket(&mut lower_bracket_rounds);
    let Some(loser_bracket_lines) = loser_bracket_lines(lower_bracket_rounds.clone()) else {
        tracing::error!("loser bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

    let (gf, gf_reset) = match dev.grand_finals_and_reset() {
        Ok((gf, bracket_reset)) => (gf, bracket_reset),
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let gf = from_participants(&gf, &participants);
    let gf_reset = from_participants(&gf_reset, &participants);

    let bracket = BracketDisplay {
        winner_bracket: winner_bracket_rounds,
        winner_bracket_lines,
        loser_bracket: lower_bracket_rounds,
        loser_bracket_lines,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
        bracket,
    };
    tracing::debug!("created bracket {:?}", bracket);
    Ok(Json(bracket))
}
