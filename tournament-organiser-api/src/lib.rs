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
    let Some(winner_bracket_lines) = winner_bracket_lines(wb_rounds.clone()) else {
        tracing::error!("winner bracket connecting lines");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    };

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
    let Some(loser_bracket_lines) = loser_bracket_lines(lb_rounds.clone()) else {
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
        winner_bracket: wb_rounds,
        winner_bracket_lines,
        loser_bracket: lb_rounds,
        loser_bracket_lines,
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

/// Display lines using boxes and their borders
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize)]
pub(crate) struct BoxElement {
    /// true when left border of box should be visible
    pub(crate) left_border: bool,
    /// true when bottom border of box should be visible
    pub(crate) bottom_border: bool,
}

// TODO refactor common code with native
/// Lines flow from matches of one round to the next round for a winner bracket
pub(crate) fn winner_bracket_lines(rounds: Vec<Vec<MinimalMatch>>) -> Option<Vec<Vec<BoxElement>>> {
    if rounds.is_empty() {
        return None;
    }

    // 3 players, 2 matches => 4
    // 4 players, 3 matches => 4
    // 5 players, 4 matches => 8
    // 6 players, 5 matches => 8
    // 9 players, 8 matches => 16
    let total_matches = rounds.iter().flatten().count();

    let Some(boxes_in_one_column) = (total_matches + 1).checked_next_power_of_two() else {
        // TODO log error
        return None;
    };
    let mut lines: Vec<Vec<BoxElement>> = vec![];

    let mut column = vec![];
    for _ in 0..boxes_in_one_column {
        column.push(BoxElement {
            left_border: false,
            bottom_border: false,
        });
    }

    // build from top to bottom (from winner bracket finals to first round)
    // start from last round and lines from previous round
    for round_index in (0..rounds.len() - 1).rev() {
        let round = &rounds[round_index];

        // FIXME remove unwrap and throw error
        let Some(matches_in_round) = (round.len()).checked_next_power_of_two() else{
            // TODO log error
            return None;
        };

        let mut left_column_flow_out_of: Vec<BoxElement> = column.clone();
        let mut right_column_flow_into: Vec<BoxElement> = column.clone();

        for (_, m) in round.iter().enumerate() {
            if let Some(row) = m.row_hint {
                let boxes_between_matches_of_same_round = boxes_in_one_column / matches_in_round;
                let Ok(r_i) = round_index.try_into() else {
                    // TODO log error
                    return None;
                };
                let Some(offset) = 2usize.checked_pow(r_i) else {
                    // TODO log error
                    return None;
                };
                // lines that flows from matches
                if total_matches == 2 {
                    left_column_flow_out_of[2].bottom_border = true;
                } else {
                    left_column_flow_out_of
                        [row * boxes_between_matches_of_same_round + offset - 1]
                        .bottom_border = true;
                }

                // vertical lines
                for j in 0..offset {
                    if row % 2 == 1 {
                        // flows down towards next match
                        right_column_flow_into[row * boxes_between_matches_of_same_round
                            + 3 * offset
                            - 1
                            - j
                            - boxes_between_matches_of_same_round]
                            .left_border = true;
                    } else {
                        // flows up towards next match
                        right_column_flow_into
                            [row * boxes_between_matches_of_same_round + 2 * offset - 1 - j]
                            .left_border = true;
                    }
                }

                if total_matches == 2 {
                    right_column_flow_into[1].bottom_border = true;
                } else if row % 2 == 1 {
                    right_column_flow_into[row * boxes_between_matches_of_same_round + offset
                        - 1
                        - boxes_between_matches_of_same_round / 2]
                        .bottom_border = true;
                }
            };
        }

        let lines_for_round = [left_column_flow_out_of, right_column_flow_into].concat();

        lines.push(lines_for_round);
    }

    // from bottom to top
    lines.reverse();

    Some(lines)
}

impl BoxElement {
    /// Box with no borders. Alternative to `default()` to use in constants
    const fn empty() -> Self {
        BoxElement {
            left_border: false,
            bottom_border: false,
        }
    }
}

/// Lines flow from matches of one round to the next round for a loser bracket
pub(crate) fn loser_bracket_lines(rounds: Vec<Vec<MinimalMatch>>) -> Option<Vec<Vec<BoxElement>>> {
    let mut lines = vec![];
    let total_matches = rounds.iter().flatten().count();

    let Some(boxes_in_one_column) = (total_matches + 1).checked_next_power_of_two() else {
        // TODO log error
        return None;
    };
    let boxes_in_one_column = boxes_in_one_column / 2;
    let mut column = vec![];
    for _ in 0..boxes_in_one_column {
        column.push(BoxElement::empty());
    }

    for (round_index, round) in rounds.iter().enumerate() {
        if round_index == rounds.len() - 1 {
            continue;
        }

        let next_round = &rounds[round_index + 1];

        // Sometimes, the first round of a loser bracket qualifies you to the
        // next round where there is more matches or the same amount of
        // matches. This the convoluted condition to draw horizontal lines from
        // one match to the other for the first round
        if round_index == 0 && rounds.len() % 2 == 0 {
            let mut left = vec![];
            let mut right = vec![];
            // we assume there may be padding matches (where row_hint is None)
            for _ in round {
                left.push(BoxElement::default());
                left.push(BoxElement::default());
                right.push(BoxElement::default());
                right.push(BoxElement::default());
            }

            // draw an horizontal line from a real matches to the next round
            for m in round {
                // Only real matches have row_hint set
                if let Some(hint) = m.row_hint {
                    left[hint * 2].bottom_border = true;
                    right[hint * 2].bottom_border = true;
                }
            }

            let straight_lines = [left, right].concat();

            lines.push(straight_lines);
        } else if round.len() == next_round.len() {
            // when there is the same amount of matches from one round to the
            // next, draw horizontal lines
            let mut straight_lines = vec![];
            for _m in round {
                straight_lines.push(BoxElement {
                    left_border: false,
                    bottom_border: true,
                });
                straight_lines.push(BoxElement::default());
                straight_lines.push(BoxElement {
                    left_border: false,
                    bottom_border: true,
                });
                straight_lines.push(BoxElement::default());
            }
            lines.push(straight_lines);
        } else {
            // when it's not the first round, either there is the same amount
            // of matches from this round to the next or there is not
            let round = &rounds[round_index];

            // FIXME remove unwrap and throw error
            // FIXME this should not be named matches_in_round but then what?
            // FIXME change name in winner bracket lines implementation also
            let Some(matches_in_round) = (round.len()).checked_next_power_of_two() else {
                // TODO log error
                return None;
            };

            // FIXME change name in winner bracket lines implementation also
            let mut left_column_flowing_out_of: Vec<BoxElement> = column.clone();
            // one or two matches flows into match
            // FIXME change name in winner bracket lines implementation also
            let mut right_column_flow_into: Vec<BoxElement> = column.clone();

            for (i, m) in round.iter().enumerate() {
                // ignore padding matches by selecting matches with set row_hint
                if let Some(row) = m.row_hint {
                    let boxes_between_matches_of_same_round =
                        boxes_in_one_column / matches_in_round;
                    // FIXME throw error
                    // Taken from winner bracket lines function. Has twice as
                    // many rounds, so gotta adjust it by dividing round_index
                    // by two
                    let Ok(r_i) =(round_index / 2).try_into() else {
                        // TODO log error
                        return None;
                    };
                    let Some(offset) = 2usize.checked_pow(r_i) else {
                        // TODO log error
                        return None;
                    };
                    if total_matches == 2 {
                        // TODO check if this branch is used (insert panic and test the case)
                        left_column_flowing_out_of[2].bottom_border = true;
                    } else {
                        left_column_flowing_out_of
                            [row * boxes_between_matches_of_same_round + offset - 1]
                            .bottom_border = true;
                    }

                    // vertical line
                    for j in 0..offset {
                        if row % 2 == 1 {
                            // flows down towards next match
                            right_column_flow_into[row * boxes_between_matches_of_same_round
                                + 3 * offset
                                - 1
                                - j
                                - boxes_between_matches_of_same_round]
                                .left_border = true;
                        } else {
                            // flows up towards next match
                            right_column_flow_into
                                [row * boxes_between_matches_of_same_round + 2 * offset - 1 - j]
                                .left_border = true;
                        }
                    }

                    if total_matches == 2 {
                        right_column_flow_into[1].bottom_border = true;
                    } else if row % 2 == 1 {
                        right_column_flow_into[row * boxes_between_matches_of_same_round
                            + offset
                            - 1
                            - boxes_between_matches_of_same_round / 2]
                            .bottom_border = true;
                    } else if i % 2 == 1 {
                        // row % 2 == 0
                        right_column_flow_into[row * boxes_between_matches_of_same_round
                            + offset
                            + 1
                            - boxes_between_matches_of_same_round / 2]
                            .bottom_border = true;
                    }
                };
            }

            let lines_for_round = [left_column_flowing_out_of, right_column_flow_into].concat();

            lines.push(lines_for_round);
        }
    }
    Some(lines)
}
