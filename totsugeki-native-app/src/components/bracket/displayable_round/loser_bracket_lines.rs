//! Lines between loser bracket rounds
use super::BoxElement;
use crate::MinimalMatch;

/// Lines flow from matches of one round to the next round for a loser bracket
pub(crate) fn lines(rounds: Vec<Vec<MinimalMatch>>) -> Option<Vec<Vec<BoxElement>>> {
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

#[cfg(test)]
mod tests {
    use super::lines;
    use crate::{
        components::bracket::displayable_round::BoxElement, ordering::loser_bracket::reorder,
        MinimalMatch,
    };

    const LINES_TO_LOSER_FINALS: [BoxElement; 4] = [
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
    ];
    const LINES_TO_SEMI_FINALS_1_OF_2: [BoxElement; 8] = [
        BoxElement::empty(),
        BoxElement::empty(),
        BoxElement {
            left_border: false,
            bottom_border: true,
        },
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
    ];
    const LINES_TO_SEMI_FINALS: [BoxElement; 8] = [
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
    fn _3_players_bracket() {
        let mut rounds = [vec![MinimalMatch::new([2, 3])]];
        reorder(&mut rounds);

        let lines = lines(rounds.to_vec()).expect("lines");

        assert_eq!(lines.len(), 0)
    }

    #[test]
    fn _4_players_bracket() {
        let mut rounds = [
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec()).expect("lines");

        assert_eq!(lines.len(), 1);
        assert_eq!(lines, vec![LINES_TO_LOSER_FINALS]);
    }

    #[test]
    fn _5_players_bracket() {
        let mut rounds = [
            vec![MinimalMatch::new([4, 5]), MinimalMatch::default()],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec()).expect("lines");

        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines,
            vec![
                LINES_TO_SEMI_FINALS_1_OF_2.to_vec(),
                LINES_TO_LOSER_FINALS.to_vec()
            ]
        );
    }

    #[test]
    fn _6_players_bracket() {
        // TODO refactor test constants in separate file so it's easier to import
        let mut rounds = [
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec()).expect("lines");

        assert_eq!(lines.len(), 2);
        assert_eq!(
            lines,
            vec![
                LINES_TO_SEMI_FINALS.to_vec(),
                LINES_TO_LOSER_FINALS.to_vec()
            ]
        );
    }
    #[test]
    fn _7_players_bracket() {
        // TODO refactor test constants in separate file so it's easier to import
        let mut rounds = [
            vec![MinimalMatch::new([6, 7])],
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        let lines = lines(rounds.to_vec()).expect("lines");

        assert_eq!(lines.len(), 3);
        assert_eq!(
            lines,
            vec![
                vec![
                    BoxElement {
                        left_border: false,
                        bottom_border: true
                    },
                    BoxElement::default(),
                    BoxElement::default(),
                    BoxElement::default(),
                    BoxElement {
                        left_border: false,
                        bottom_border: true
                    },
                    BoxElement::default(),
                    BoxElement::default(),
                    BoxElement::default(),
                ],
                LINES_TO_SEMI_FINALS.to_vec(),
                LINES_TO_LOSER_FINALS.to_vec(),
            ]
        );
    }
}
