//! Lines between loser bracket rounds
use super::BoxWithBorder;
use crate::DisplayableMatch;

/// Lines flow from matches of one round to the next round for a loser bracket
pub(crate) fn lines(rounds: Vec<Vec<DisplayableMatch>>) -> Vec<Vec<BoxWithBorder>> {
    let mut lines = vec![];
    let total_matches = rounds.iter().flatten().count();

    let boxes_in_one_column = (total_matches + 1).checked_next_power_of_two().unwrap();
    let boxes_in_one_column = boxes_in_one_column / 2;
    println!("boxes: {}", boxes_in_one_column); // FIXME remove
    let mut column = vec![];
    for _ in 0..boxes_in_one_column {
        column.push(BoxWithBorder {
            left: false,
            bottom: false,
        });
    }

    for (round_index, round) in rounds.iter().enumerate() {
        if round_index == rounds.len() - 1 {
            continue;
        }

        let next_round = &rounds[round_index + 1];
        // FIXME straight lines for when the number of matches between rounds is not the same

        if round_index == 0 && rounds.len() % 2 == 0 {
            let mut straight_lines = vec![];
            let mut left = vec![];
            let mut right = vec![];
            for _m in round {
                left.push(BoxWithBorder {
                    left: false,
                    bottom: false,
                });
                left.push(BoxWithBorder {
                    left: false,
                    bottom: false,
                });
                right.push(BoxWithBorder {
                    left: false,
                    bottom: false,
                });
                right.push(BoxWithBorder {
                    left: false,
                    bottom: false,
                });
            }
            // border bottom
            for m in round {
                // TODO change left and right vectors
                if let Some(hint) = m.row_hint {
                    left[hint * 2].bottom = true;
                    right[hint * 2].bottom = true;
                }
            }
            straight_lines = [left, right].concat();

            lines.push(straight_lines);
        } else if round.len() == next_round.len() {
            let mut straight_lines = vec![];
            for m in round {
                straight_lines.push(BoxWithBorder {
                    left: false,
                    bottom: true,
                });
                straight_lines.push(BoxWithBorder {
                    left: false,
                    bottom: false,
                });
                straight_lines.push(BoxWithBorder {
                    left: false,
                    bottom: true,
                });
                straight_lines.push(BoxWithBorder {
                    left: false,
                    bottom: false,
                });
            }
            lines.push(straight_lines);
        } else {
            let round = &rounds[round_index];

            // FIXME remove unwrap and throw error
            // FIXME this should not be named matches_in_round but then what?
            // FIXME change name in winner bracket lines implementation also
            let matches_in_round = (round.len()).checked_next_power_of_two().unwrap();

            // FIXME change name in winner bracket lines implementation also
            let mut left_column_flowing_out_of: Vec<BoxWithBorder> = column.clone();
            // one or two matches flows into match
            // FIXME change name in winner bracket lines implementation also
            let mut right_column_flow_into: Vec<BoxWithBorder> = column.clone();

            for (i, m) in round.iter().enumerate() {
                // ignore padding matches by selecting matches with set row_hint
                if let Some(row) = m.row_hint {
                    let boxes_between_matches_of_same_round =
                        boxes_in_one_column / matches_in_round;
                    let offset = 2usize
                        .checked_pow((round_index / 2).try_into().unwrap())
                        .unwrap();
                    if total_matches == 2 {
                        left_column_flowing_out_of[2].bottom = true;
                    } else {
                        left_column_flowing_out_of
                            [row * boxes_between_matches_of_same_round + offset - 1]
                            .bottom = true;
                    }

                    // vertical line
                    for j in 0..offset {
                        if row % 2 == 1 {
                            // flows down towards next match
                            // FIXME uncomment and fix when not straight lines
                            right_column_flow_into[row * boxes_between_matches_of_same_round
                                + 3 * offset
                                - 1
                                - j
                                - boxes_between_matches_of_same_round]
                                .left = true;
                        } else {
                            // flows up towards next match
                            right_column_flow_into
                                [row * boxes_between_matches_of_same_round + 2 * offset - 1 - j]
                                .left = true;
                        }
                    }

                    if total_matches == 2 {
                        right_column_flow_into[1].bottom = true;
                    } else if row % 2 == 1 {
                        right_column_flow_into[row * boxes_between_matches_of_same_round
                            + offset
                            - 1
                            - boxes_between_matches_of_same_round / 2]
                            .bottom = true;
                    } else if i % 2 == 1 {
                        // row % 2 == 0
                        right_column_flow_into[row * boxes_between_matches_of_same_round
                            + offset
                            + 1
                            - boxes_between_matches_of_same_round / 2]
                            .bottom = true;
                    }
                };
            }

            let lines_for_round = [left_column_flowing_out_of, right_column_flow_into].concat();

            lines.push(lines_for_round);
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use crate::{
        components::bracket::displayable_round::BoxWithBorder, ordering::loser_bracket::reorder,
        DisplayableMatch,
    };

    use super::lines;

    #[test]
    fn _3_players_bracket() {
        let mut rounds = [vec![DisplayableMatch::new([2, 3])]];
        reorder(&mut rounds);

        let lines = lines(rounds.to_vec());

        assert_eq!(lines.len(), 0)
    }

    #[test]
    fn _4_players_bracket() {
        // TODO refactor test constants in separate file so it's easier to import
        let mut rounds = [
            vec![DisplayableMatch::new([3, 4])],
            vec![DisplayableMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec());

        assert_eq!(lines.len(), 1);
        assert_eq!(
            lines,
            vec![vec![
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
            ]]
        );
    }

    #[test]
    fn _5_players_bracket() {
        // TODO refactor test constants in separate file so it's easier to import
        let mut rounds = [
            vec![DisplayableMatch::new([4, 5])],
            vec![DisplayableMatch::new([3, 4])],
            vec![DisplayableMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec());

        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines,
            vec![
                vec![
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
                ],
                vec![
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
                ],
            ]
        );
    }

    #[test]
    fn _6_players_bracket() {
        // TODO refactor test constants in separate file so it's easier to import
        let mut rounds = [
            vec![DisplayableMatch::new([3, 6]), DisplayableMatch::new([4, 5])],
            vec![DisplayableMatch::new([3, 4])],
            vec![DisplayableMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec());

        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines,
            vec![
                vec![
                    BoxWithBorder {
                        left: false,
                        bottom: true,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false,
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: true,
                    },
                    BoxWithBorder {
                        left: true,
                        bottom: false,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false,
                    },
                ],
                vec![
                    BoxWithBorder {
                        left: false,
                        bottom: true,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: true,
                    },
                    BoxWithBorder {
                        left: false,
                        bottom: false,
                    },
                ],
            ]
        );
    }
    #[test]
    fn _7_players_bracket() {
        // TODO refactor test constants in separate file so it's easier to import
        let mut rounds = [
            vec![DisplayableMatch::new([6, 7])],
            vec![DisplayableMatch::new([3, 6]), DisplayableMatch::new([4, 5])],
            vec![DisplayableMatch::new([3, 4])],
            vec![DisplayableMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec());

        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines,
            vec![
                vec![
                    // TODO adjust for first round
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
                ],
                vec![
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
                ],
                vec![
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
                ],
                vec![
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
                ],
            ]
        );
    }
}
