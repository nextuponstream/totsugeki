//! Lines of winner bracket

// TODO add 16+17 players bracket test

#[cfg(test)]
mod tests {
    use totsugeki::bracket::seeding::Seeding;
    use totsugeki::player::PlayerID;
    use totsugeki::single_elimination_bracket::SingleEliminationBracket;

    fn get_data(n: usize) -> SingleEliminationBracket {
        let mut seeding = vec![];
        for _ in 0..n {
            seeding.push(PlayerID::create());
        }
        SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true)
    }

    use crate::from_participants;
    use crate::winner_bracket::lines;
    use crate::winner_bracket::reorder;
    use crate::BoxElement;

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
        let players = bracket
            .get_seeding()
            .get()
            .iter()
            .map(|id| (*id, "p").try_into().unwrap())
            .collect::<Vec<totsugeki::player::Player>>();
        let matches_by_rounds = bracket.partition_by_round().expect("rounds");
        let mut rounds = vec![];
        for r in matches_by_rounds {
            let round = r.iter().map(|m| from_participants(m, &players)).collect();
            rounds.push(round);
        }

        reorder(&mut rounds);

        let lines = lines(&rounds).expect("lines");
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
        let players = bracket
            .get_seeding()
            .get()
            .iter()
            .map(|id| (*id, "p").try_into().unwrap())
            .collect::<Vec<totsugeki::player::Player>>();
        let matches_by_rounds = bracket.partition_by_round().expect("rounds");
        let mut rounds = vec![];
        for r in matches_by_rounds {
            let round = r.iter().map(|m| from_participants(m, &players)).collect();
            rounds.push(round);
        }
        reorder(&mut rounds);

        let lines = lines(&rounds).expect("lines");
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
        let players = bracket
            .get_seeding()
            .get()
            .iter()
            .map(|id| (*id, "p").try_into().unwrap())
            .collect::<Vec<totsugeki::player::Player>>();
        let matches_by_rounds = bracket.partition_by_round().expect("rounds");
        let mut rounds = vec![];
        for r in matches_by_rounds {
            let round = r.iter().map(|m| from_participants(m, &players)).collect();
            rounds.push(round);
        }
        reorder(&mut rounds);

        let lines = lines(&rounds).expect("lines");
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
