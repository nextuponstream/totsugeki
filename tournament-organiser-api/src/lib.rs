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
    winner_bracket: Vec<Vec<MinimalMatch>>,
    /// Loser bracket matches and lines to draw
    loser_bracket: Vec<Vec<MinimalMatch>>,
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
    let participants = bracket.get_participants();
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    // TODO test if tracing shows from which methods it was called
    let wb_rounds_matches = match dev.partition_winner_bracket() {
        Ok(wb) => wb,
        Err(e) => {
            tracing::error!("{e:?}");
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    };
    let mut wb_rounds = vec![];
    for r in wb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        wb_rounds.push(round);
    }

    reorder(&mut wb_rounds);

    let Ok(lb_rounds_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };
    let mut lb_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        lb_rounds.push(round);
    }
    reorder_loser_bracket(&mut lb_rounds);

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
        winner_bracket: wb_rounds,
        loser_bracket: lb_rounds,
        grand_finals: gf,
        grand_finals_reset: gf_reset,
    };
    tracing::debug!("created bracket {:?}", bracket);
    Ok(Json(bracket))
}

// TODO use common lib between native and tournament-organiser-api
/// Give positionnal hints to winner bracket matches
pub fn reorder(rounds: &mut [Vec<MinimalMatch>]) {
    if rounds.len() < 2 {
        return;
    }

    // set hint for all rounds except last two
    // traverse from last to first
    for i in (0..rounds.len() - 2).rev() {
        let mut round = rounds[i].clone();
        let number_of_matches_in_round = rounds[i + 1].len() * 2;

        // iterate over previous round and set positional hints
        for (j, m) in rounds[i + 1].iter().enumerate() {
            let row_hint_1 = 2 * j;
            let row_hint_2 = 2 * j + 1;

            let seed_1 = m.seeds[0];
            // (first) player of round 1 with highest seed is expected to win
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == seed_1) {
                m.row_hint = Some(row_hint_1);
            }
            let seed_2 = m.seeds[1];
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == seed_2) {
                m.row_hint = Some(row_hint_2);
            }
        }

        // Padding matches for initial round
        if i == 0 {
            for _ in 0..number_of_matches_in_round - rounds[i].len() {
                round.push(MinimalMatch::default())
            }
        }

        // sort row i+1 so unsorted row i can be sorted next iteration
        // NOTE: for round 1, filler matches are first after sorting
        round.sort_by_key(|m| m.row_hint);
        rounds[i] = round;
    }

    // round before last round
    rounds[rounds.len() - 2][0].row_hint = Some(0);
    // when there is exactly 3 players
    if rounds[rounds.len() - 2].len() == 1 {
        rounds[rounds.len() - 2][0].row_hint = Some(1);
    } else {
        rounds[rounds.len() - 2][1].row_hint = Some(1);
    }

    // last round
    rounds[rounds.len() - 1][0].row_hint = Some(0);
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

impl Default for MinimalMatch {
    fn default() -> Self {
        MinimalMatch {
            id: MatchId::new_v4(),
            players: [String::default(), String::default()],
            score: (0, 0),
            seeds: [0, 0],
            row_hint: None,
        }
    }
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

// TODO refactor common code in native
/// Give positionnal hints to loser bracket matches
pub fn reorder_loser_bracket(rounds: &mut [Vec<MinimalMatch>]) {
    if rounds.len() < 2 {
        return;
    }

    let lb_rounds_count = rounds.len();

    // give row hints to last 3 rounds
    if lb_rounds_count > 2 {
        rounds[lb_rounds_count - 3][0].row_hint = Some(0);
        if rounds[lb_rounds_count - 3].len() > 1 {
            rounds[lb_rounds_count - 3][1].row_hint = Some(1);
        }
    }

    if lb_rounds_count > 1 {
        rounds[lb_rounds_count - 2][0].row_hint = Some(0);
    }

    rounds[lb_rounds_count - 1][0].row_hint = Some(0);

    // give hints to all other rounds
    for i in (0..rounds.len() - 2).rev() {
        let mut round = rounds[i].clone();
        let number_of_matches_in_round = rounds[i + 1].len() * 2;

        // iterate over previous round and set positional hints
        for (j, m) in rounds[i + 1].iter().enumerate() {
            let winner_seed = m.seeds[0];
            // (first) player of round 1 with highest seed is expected to win
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == winner_seed) {
                m.row_hint = Some(rounds[i + 1][j].row_hint.expect("") * 2);
            }
            let loser_seed = m.seeds[1];
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == loser_seed) {
                if (lb_rounds_count - i) % 2 == 0 {
                    m.row_hint = rounds[i + 1][j].row_hint;
                } else {
                    // 7-10 (1), 8-9 (3)
                    m.row_hint = Some(rounds[i + 1][j].row_hint.expect("") * 2 + 1);
                }
            }
        }

        if i == 0 {
            if rounds.len() % 2 == 0 {
                for _ in 0..rounds[i + 1].len() - rounds[i].len() {
                    round.push(MinimalMatch::default())
                }
            } else {
                for _ in 0..number_of_matches_in_round - rounds[i].len() {
                    round.push(MinimalMatch::default())
                }
            }
        }

        round.sort_by_key(|m| m.row_hint);
        rounds[i] = round;
    }
}
