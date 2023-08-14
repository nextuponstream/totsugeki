//! test lines to display for loser bracket

#[cfg(test)]
mod tests {
    use crate::loser_bracket::lines;
    use crate::loser_bracket::reorder;
    use crate::BoxElement;
    use crate::MinimalMatch;

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

        assert_eq!(lines.len(), 0);
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
