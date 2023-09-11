//! Lines between rounds of a winner bracket

use super::BoxElement;
use crate::MinimalMatch;

/// Lines flow from matches of one round to the next round for a winner bracket
pub(crate) fn lines(rounds: Vec<Vec<MinimalMatch>>) -> Option<Vec<Vec<BoxElement>>> {
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

        let Some(matches_in_round) = (round.len()).checked_next_power_of_two() else {
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

#[cfg(test)]
mod tests {
    use chrono::prelude::*;
    use totsugeki::bracket::single_elimination_variant::Variant;
    use totsugeki::bracket::Bracket;

    fn get_data(n: usize) -> Bracket {
        let mut bracket = Bracket::new(
            "",
            totsugeki::format::Format::SingleElimination,
            totsugeki::seeding::Method::Strict,
            DateTime::default(),
            true,
        );
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("player {i}").as_str())
                .expect("bracket");
        }

        bracket
    }

    use super::{lines, BoxElement};
    use crate::from_participants;
    use crate::ordering::winner_bracket::reorder;

    const LINES_TO_WINNERS_FINALS: [BoxElement; 8] = [
        // left col
        BoxElement {
            left_border: false,
            bottom_border: true,
        },
        BoxElement::empty(),
        BoxElement {
            left_border: false,
            bottom_border: true,
        },
        BoxElement::empty(),
        // right col
        BoxElement::empty(),
        BoxElement {
            left_border: true,
            bottom_border: true,
        },
        BoxElement {
            left_border: true,
            bottom_border: false,
        },
        BoxElement::empty(),
    ];

    #[test]
    fn _3_participants_bracket() {
        let bracket = get_data(3);
        let participants = bracket.get_participants();
        let sev: Variant = bracket.try_into().expect("single elimination bracket");
        let matches_by_rounds = sev.partition_by_round().expect("rounds");
        let mut rounds = vec![];
        for r in matches_by_rounds {
            let round = r
                .iter()
                .map(|m| from_participants(m, &participants))
                .collect();
            rounds.push(round)
        }

        reorder(&mut rounds);

        let lines = lines(rounds.clone()).expect("lines");
        let expected_cols = 1;
        assert_eq!(lines.len(), expected_cols);
        //     b1L1   b1R1
        //
        //     b1L2   b1R2
        //          --------> m3
        //     b1L3 | b1R3
        // m1 ------
        //     b1L4   b1R4
        assert_eq!(
            lines,
            vec![
                // col 1
                vec![
                    // left col
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement {
                        left_border: false,
                        bottom_border: true
                    },
                    BoxElement::empty(),
                    // right col
                    BoxElement::empty(),
                    BoxElement {
                        left_border: false,
                        bottom_border: true
                    },
                    BoxElement {
                        left_border: true,
                        bottom_border: false
                    },
                    BoxElement::empty(),
                ]
            ]
        );
    }

    #[test]
    fn _4_participants_bracket() {
        let bracket = get_data(4);
        let participants = bracket.get_participants();
        let sev: Variant = bracket.try_into().expect("single elimination bracket");
        let matches_by_rounds = sev.partition_by_round().expect("rounds");
        let mut rounds = vec![];
        for r in matches_by_rounds {
            let round = r
                .iter()
                .map(|m| from_participants(m, &participants))
                .collect();
            rounds.push(round)
        }
        reorder(&mut rounds);

        let lines = lines(rounds.clone()).expect("lines");
        let expected_cols = 1;
        assert_eq!(lines.len(), expected_cols);
        //     b1L1   b1R1
        // m1 -------
        //     b1L2 | b1R2
        //          --------> m3
        //     b1L3 | b1R3
        // m2 ------
        //     b1L4   b1R4
        assert_eq!(lines, vec![LINES_TO_WINNERS_FINALS,]);
    }

    #[test]
    fn _5_participants_bracket() {
        let bracket = get_data(5);
        let participants = bracket.get_participants();
        let sev: Variant = bracket.try_into().expect("single elimination bracket");
        let matches_by_rounds = sev.partition_by_round().expect("rounds");
        let mut rounds = vec![];
        for r in matches_by_rounds {
            let round = r
                .iter()
                .map(|m| from_participants(m, &participants))
                .collect();
            rounds.push(round)
        }
        reorder(&mut rounds);

        let lines = lines(rounds.clone()).expect("lines");
        let expected_cols = 2;
        assert_eq!(lines.len(), expected_cols);
        //     b1L1   b1R1
        // m1 -------
        //     b1L2 | b1R2
        //          --------> m3
        //     b1L3 | b1R3
        // m2 ------
        //     b1L4   b1R4
        assert_eq!(
            lines,
            vec![
                vec![
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement {
                        left_border: false,
                        bottom_border: true,
                    },
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement::empty(),
                    // right
                    BoxElement::empty(),
                    BoxElement {
                        left_border: false,
                        bottom_border: true,
                    },
                    BoxElement {
                        left_border: true,
                        bottom_border: false,
                    },
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement::empty(),
                ],
                vec![
                    BoxElement::empty(),
                    BoxElement {
                        left_border: false,
                        bottom_border: true,
                    },
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement {
                        left_border: false,
                        bottom_border: true,
                    },
                    BoxElement::empty(),
                    BoxElement::empty(),
                    // right col
                    BoxElement::empty(),
                    BoxElement::empty(),
                    BoxElement {
                        left_border: true,
                        bottom_border: false,
                    },
                    BoxElement {
                        left_border: true,
                        bottom_border: true,
                    },
                    BoxElement {
                        left_border: true,
                        bottom_border: false,
                    },
                    BoxElement {
                        left_border: true,
                        bottom_border: false,
                    },
                    BoxElement::empty(),
                    BoxElement::empty(),
                ]
            ]
        );
    }

    // TODO add test cases for many to help if someone refactors this later
}
