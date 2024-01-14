//! Display winner bracket

mod test_lines;
mod test_ordering;

use crate::{BoxElement, MinimalMatch};

/// Lines flow from matches of one round to the next round for a winner bracket
#[must_use]
pub fn lines(rounds: &[Vec<MinimalMatch>]) -> Option<Vec<Vec<BoxElement>>> {
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
        let Some(matches_in_round) = (round.len()).checked_next_power_of_two() else {
            // TODO log error
            return None;
        };

        let mut left_column_flow_out_of: Vec<BoxElement> = column.clone();
        let mut right_column_flow_into: Vec<BoxElement> = column.clone();

        for m in round {
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

/// Set positionnal hints to winner bracket matches
// TODO rename to explain Tailwind CSS dependency
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
                round.push(MinimalMatch::default());
            }
        }

        // sort row i so unsorted row i-1 can be sorted next iteration
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
