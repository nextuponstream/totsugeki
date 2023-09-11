//! Display loser bracket

use crate::{BoxElement, MinimalMatch};

mod test_lines;
mod test_ordering;

/// Give positionnal hints to loser bracket matches
///
/// # Panics
/// When provided matches do no give row hints
pub fn reorder(rounds: &mut [Vec<MinimalMatch>]) {
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
                m.row_hint = Some(rounds[i + 1][j].row_hint.expect("row hint") * 2);
            }
            let loser_seed = m.seeds[1];
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == loser_seed) {
                if (lb_rounds_count - i) % 2 == 0 {
                    m.row_hint = rounds[i + 1][j].row_hint;
                } else {
                    // 7-10 (1), 8-9 (3)
                    m.row_hint = Some(rounds[i + 1][j].row_hint.expect("row hint") * 2 + 1);
                }
            }
        }

        if i == 0 {
            if rounds.len() % 2 == 0 {
                for _ in 0..rounds[i + 1].len() - rounds[i].len() {
                    round.push(MinimalMatch::default());
                }
            } else {
                for _ in 0..number_of_matches_in_round - rounds[i].len() {
                    round.push(MinimalMatch::default());
                }
            }
        }

        round.sort_by_key(|m| m.row_hint);
        rounds[i] = round;
    }
}

/// Lines flow from matches of one round to the next round for a loser bracket
// FIXME add tests in this crate
// FIXME remove allow macros
#[must_use]
#[allow(clippy::needless_pass_by_value)]
#[allow(clippy::too_many_lines)]
pub fn lines(rounds: Vec<Vec<MinimalMatch>>) -> Option<Vec<Vec<BoxElement>>> {
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
                    let Ok(r_i) = (round_index / 2).try_into() else {
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
