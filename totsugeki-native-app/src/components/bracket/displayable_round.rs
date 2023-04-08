//! Display some round
#![allow(non_snake_case)]

use crate::components::bracket::displayable_match::DisplayMatch;
use crate::DisplayableMatch;
use dioxus::prelude::*;

pub(crate) fn Round(cx: Scope, round: Vec<DisplayableMatch>) -> Element {
    cx.render(rsx!(
        div {
            class: "grid grid-cols-1",
            round.iter().map(|m| DisplayMatch(cx, *m))
        }
    ))
}

/// Box that may have a left or bottom border
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct BoxWithBorder {
    pub(crate) left: bool,
    pub(crate) bottom: bool,
}

/// Lines flow from matches of one round to the next round for a winner bracket
pub(crate) fn winner_bracket_lines(rounds: Vec<Vec<DisplayableMatch>>) -> Vec<Vec<BoxWithBorder>> {
    if rounds.is_empty() {
        return vec![];
    }
    // 3 players, 2 matches => 4
    // 4 players, 3 matches => 4
    // 5 players, 4 matches => 8
    // 6 players, 5 matches => 8
    // 9 players, 8 matches => 16
    let total_matches = rounds.iter().flatten().count();
    let boxes_in_one_column = (total_matches + 1).checked_next_power_of_two().unwrap();
    println!("boxes in one column: {}", boxes_in_one_column); // FIXME remove
    let mut lines: Vec<Vec<BoxWithBorder>> = vec![];

    // b belongs in [1; #matches in current round]
    let mut column = vec![];
    for _ in 0..boxes_in_one_column {
        column.push(BoxWithBorder {
            left: false,
            bottom: false,
        });
    }

    // build from top to bottom (from winner bracket finals to first round)
    // start from last round and lines from previous round
    let mut late_round = 0;
    for round_index in (0..rounds.len() - 1).rev() {
        println!("round index: {round_index}"); // FIXME remove
        let round = &rounds[round_index];

        // println!("{:?}", round);
        let matches_in_round = (round.len()).checked_next_power_of_two().unwrap();
        println!("matches in round: {matches_in_round}");

        let mut left_column: Vec<BoxWithBorder> = column.clone();
        let mut right_column: Vec<BoxWithBorder> = column.clone();

        for (_, m) in round.iter().enumerate() {
            if let Some(row) = m.row_hint {
                // left line
                // if m X present, draw bottom border of b * 2
                // FIXME does not work for 9 player

                // n = 3
                // m1
                // 4 boxes in 1 col
                // ... = 1
                // m2
                // ... = 2
                let boxes_between_matches_of_same_round = boxes_in_one_column / matches_in_round;
                println!("row: {row}");
                println!("bva: {}", boxes_between_matches_of_same_round);
                let offset = 2usize.checked_pow(round_index.try_into().unwrap()).unwrap();
                println!("off: {}", offset);
                println!(
                    "row*bva+rni={}",
                    row * boxes_between_matches_of_same_round + offset - 1
                );
                println!("---");

                // FIXME n = 3
                left_column[row * boxes_between_matches_of_same_round + offset - 1].bottom = true;

                // vertical line
                // if m X present, draw left border of b * 2 + 1 until ???
                for j in 0..offset {
                    if row % 2 == 1 {
                        // flows down towards next match
                        // right_column[row * boxes_between_matches_of_same_round + offset - 1 + j]
                        // .left = true;
                        right_column[row * boxes_between_matches_of_same_round + 3 * offset
                            - 1
                            - j
                            - boxes_between_matches_of_same_round]
                            .left = true;
                    } else {
                        // flows up towards next match
                        right_column
                            [row * boxes_between_matches_of_same_round + 2 * offset - 1 - j]
                            .left = true;
                    }
                }

                // right light
                // might be set to true two times (which is slightly innefficient)
                // example: 4 players, m1 and m2 need to flow into m3
                // TODO test if only need to draw once since filling a match
                // always fills lower seed, then higher seed if needed
                // draw bottom border of b * 2
                if row % 2 == 1 {
                    right_column[row * boxes_between_matches_of_same_round + offset
                        - 1
                        - boxes_between_matches_of_same_round / 2]
                        .bottom = true;
                }
            };
        }

        let lines_for_round = [left_column, right_column].concat();

        lines.push(lines_for_round);
    }

    // from bottom to top
    lines.reverse();

    lines
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use totsugeki::{bracket::Bracket, matches::Match, player::Participants};

    use crate::single_elimination::{ordering::reorder_rounds, partition::winner_bracket};

    use super::{winner_bracket_lines, BoxWithBorder};

    fn get_data(n: usize) -> (Vec<Match>, Participants) {
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
                .unwrap();
        }
        let matches = bracket.get_matches();
        let participants = bracket.get_participants();
        (matches, participants)
    }

    #[test]
    fn _3_participants_bracket() {
        let (matches, participants) = get_data(3);
        let mut rounds = winner_bracket(matches, &participants);
        reorder_rounds(&mut rounds);

        let lines = winner_bracket_lines(rounds.clone());
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
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    // right col
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    }
                ]
            ]
        );
    }

    #[test]
    fn _4_participants_bracket() {
        let (matches, participants) = get_data(4);
        let mut rounds = winner_bracket(matches, &participants);
        reorder_rounds(&mut rounds);

        let lines = winner_bracket_lines(rounds.clone());
        let expected_cols = 1;
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
                // col 1
                vec![
                    // left col
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    // right col
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: true,
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                ],
            ]
        );
    }
    #[test]
    fn _5_participants_bracket() {
        let (matches, participants) = get_data(5);
        let mut rounds = winner_bracket(matches, &participants);
        reorder_rounds(&mut rounds);

        let lines = winner_bracket_lines(rounds.clone());
        let expected_cols = 2;
        assert_eq!(lines.len(), expected_cols);
        //    b1L1   b1R1       b2L1   b2R1
        //
        //    b1L2   b1R2       b2L2   b2R2
        //         -------> m2 ------|
        //    b1L3 | b1R3       b2L3 | b2R3
        // m1 -----|                 |
        //    b1L4   b1R4       b2L4 | b2R4
        //                           ----------> m4
        //    b1L5   b1R5       b2L5 | b2R5
        //                           |
        //    b1L6   b1R6       b2L6 | b2R6
        //                  m3 ------|
        //    b1L7   b1R7       b2L7   b2R7
        //
        //    b1L8   b1R8       b2L8   b2R8
        assert_eq!(
            lines,
            vec![
                // col 1
                vec![
                    // left col
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    // right col
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                ],
                // col2
                vec![
                    // left col
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    // right col
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: true
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false
                    },
                ],
            ]
        );
    }
}
