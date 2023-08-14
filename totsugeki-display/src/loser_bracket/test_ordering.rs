//! test ordering of loser bracket matches

#[cfg(test)]
mod tests {
    use crate::MinimalMatch;

    use crate::loser_bracket::reorder;

    #[test]
    fn _3_players() {
        let mut rounds = [vec![MinimalMatch::new([2, 3])]];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 1);
        assert_eq!(rounds[0][0].row_hint, None);
    }

    #[test]
    fn _4_players() {
        let mut rounds = [
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 2);
        assert_eq!(rounds[0][0].row_hint, Some(0));
        assert_eq!(rounds[1][0].row_hint, Some(0));
    }

    #[test]
    fn _5_players() {
        let mut rounds = [
            vec![MinimalMatch::new([4, 5]), MinimalMatch::default()],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 3);
        assert_eq!(
            rounds[0][0].row_hint,
            Some(1),
            "4-5 {:?}",
            rounds[0][0].summary()
        );
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[2][0].row_hint, Some(0));
    }

    #[test]
    fn _6_players() {
        let mut rounds = [
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 3);
        assert_eq!(rounds[0][0].row_hint, Some(0), "{}", rounds[0][0].summary());
        assert_eq!(rounds[0][1].row_hint, Some(1));
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[2][0].row_hint, Some(0));
    }

    #[test]
    fn _8_players() {
        let mut rounds = [
            vec![MinimalMatch::new([5, 8]), MinimalMatch::new([6, 7])],
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 4);
        assert_eq!(rounds[3][0].row_hint, Some(0));
        assert_eq!(rounds[2][0].row_hint, Some(0));
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[1][1].row_hint, Some(1));
        assert_eq!(
            rounds[0][0].row_hint,
            Some(0),
            "{:?}",
            rounds[0][0].summary()
        );
        assert_eq!(
            rounds[0][1].row_hint,
            Some(1),
            "{:?}",
            rounds[0][1].summary()
        );
    }

    #[test]
    fn _9_players() {
        let mut rounds = [
            vec![
                MinimalMatch::new([8, 9]),
                MinimalMatch::default(),
                MinimalMatch::default(),
                MinimalMatch::default(),
            ],
            vec![MinimalMatch::new([5, 8]), MinimalMatch::new([6, 7])],
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 5);
        assert_eq!(rounds[4].len(), 1);
        assert_eq!(rounds[4][0].seeds, [2, 3]); // 2 drops in
        assert_eq!(rounds[4][0].row_hint, Some(0));

        assert_eq!(rounds[3].len(), 1);
        assert_eq!(rounds[3][0].seeds, [3, 4]); // previous rounds
        assert_eq!(rounds[3][0].row_hint, Some(0));

        assert_eq!(rounds[2].len(), 2);
        assert_eq!(rounds[2][0].seeds, [3, 6]); // 3 drops in
        assert_eq!(rounds[2][0].row_hint, Some(0));
        assert_eq!(rounds[2][1].seeds, [4, 5]); // 4 drops in
        assert_eq!(rounds[2][1].row_hint, Some(1));

        assert_eq!(rounds[1].len(), 2);
        assert_eq!(rounds[1][0].seeds, [6, 7]); // 6 drops in
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[1][1].seeds, [5, 8]); // 5 drops in
        assert_eq!(rounds[1][1].row_hint, Some(1));

        assert_eq!(rounds[0].len(), 4);
        assert_eq!(rounds[0][3].seeds, [8, 9]); // 8 and 9 drops in
        assert_eq!(rounds[0][3].row_hint, Some(3));
    }

    #[test]
    fn _10_players() {
        let mut rounds = [
            vec![
                MinimalMatch::new([7, 10]),
                MinimalMatch::new([8, 9]),
                MinimalMatch::default(),
                MinimalMatch::default(),
            ],
            vec![MinimalMatch::new([5, 8]), MinimalMatch::new([6, 7])],
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 5);
        assert_eq!(rounds[4].len(), 1);
        assert_eq!(rounds[4][0].seeds, [2, 3]); // 2 drops in
        assert_eq!(rounds[4][0].row_hint, Some(0));

        assert_eq!(rounds[3].len(), 1);
        assert_eq!(rounds[3][0].seeds, [3, 4]); // previous rounds
        assert_eq!(rounds[3][0].row_hint, Some(0));

        assert_eq!(rounds[2].len(), 2);
        assert_eq!(rounds[2][0].seeds, [3, 6]); // 3 drops in
        assert_eq!(rounds[2][0].row_hint, Some(0));
        assert_eq!(rounds[2][1].seeds, [4, 5]); // 4 drops in
        assert_eq!(rounds[2][1].row_hint, Some(1));

        assert_eq!(rounds[1].len(), 2);
        assert_eq!(rounds[1][0].seeds, [6, 7]); // 6 drops in
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[1][1].seeds, [5, 8]); // 5 drops in
        assert_eq!(rounds[1][1].row_hint, Some(1));

        assert_eq!(rounds[0].len(), 4);
        assert_eq!(rounds[0][2].seeds, [7, 10]); // 8 and 9 drops in
        assert_eq!(rounds[0][2].row_hint, Some(1));
        assert_eq!(rounds[0][3].seeds, [8, 9]); // 8 and 9 drops in
        assert_eq!(rounds[0][3].row_hint, Some(3));
    }

    #[test]
    fn _11_players() {
        let mut rounds = [
            vec![
                MinimalMatch::new([6, 11]),
                MinimalMatch::new([7, 10]),
                MinimalMatch::new([8, 9]),
                MinimalMatch::default(),
            ],
            vec![MinimalMatch::new([5, 8]), MinimalMatch::new([6, 7])],
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 5);
        assert_eq!(rounds[4].len(), 1);
        assert_eq!(rounds[4][0].seeds, [2, 3]); // 2 drops in
        assert_eq!(rounds[4][0].row_hint, Some(0));

        assert_eq!(rounds[3].len(), 1);
        assert_eq!(rounds[3][0].seeds, [3, 4]); // previous rounds
        assert_eq!(rounds[3][0].row_hint, Some(0));

        assert_eq!(rounds[2].len(), 2);
        assert_eq!(rounds[2][0].seeds, [3, 6]); // 3 drops in
        assert_eq!(rounds[2][0].row_hint, Some(0));
        assert_eq!(rounds[2][1].seeds, [4, 5]); // 4 drops in
        assert_eq!(rounds[2][1].row_hint, Some(1));

        assert_eq!(rounds[1].len(), 2);
        assert_eq!(rounds[1][0].seeds, [6, 7]); // 6 drops in
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[1][1].seeds, [5, 8]); // 5 drops in
        assert_eq!(rounds[1][1].row_hint, Some(1));

        assert_eq!(rounds[0].len(), 4);
        assert_eq!(rounds[0][1].seeds, [6, 11]); // 8 and 9 drops in
        assert_eq!(rounds[0][1].row_hint, Some(0));
        assert_eq!(rounds[0][2].seeds, [7, 10]); // 8 and 9 drops in
        assert_eq!(rounds[0][2].row_hint, Some(1));
        assert_eq!(rounds[0][3].seeds, [8, 9]); // 8 and 9 drops in
        assert_eq!(rounds[0][3].row_hint, Some(3));
    }

    #[test]
    fn _12_players() {
        let mut rounds = [
            vec![
                MinimalMatch::new([5, 12]),
                MinimalMatch::new([6, 11]),
                MinimalMatch::new([7, 10]),
                MinimalMatch::new([8, 9]),
            ],
            vec![MinimalMatch::new([5, 8]), MinimalMatch::new([6, 7])],
            vec![MinimalMatch::new([3, 6]), MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([3, 4])],
            vec![MinimalMatch::new([2, 3])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds.len(), 5);
        assert_eq!(rounds[4].len(), 1);
        assert_eq!(rounds[4][0].seeds, [2, 3]); // 2 drops in
        assert_eq!(rounds[4][0].row_hint, Some(0));

        assert_eq!(rounds[3].len(), 1);
        assert_eq!(rounds[3][0].seeds, [3, 4]); // previous rounds
        assert_eq!(rounds[3][0].row_hint, Some(0));

        assert_eq!(rounds[2].len(), 2);
        assert_eq!(rounds[2][0].seeds, [3, 6]); // 3 drops in
        assert_eq!(rounds[2][0].row_hint, Some(0));
        assert_eq!(rounds[2][1].seeds, [4, 5]); // 4 drops in
        assert_eq!(rounds[2][1].row_hint, Some(1));

        assert_eq!(rounds[1].len(), 2);
        assert_eq!(rounds[1][0].seeds, [6, 7]); // 6 drops in
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[1][1].seeds, [5, 8]); // 5 drops in
        assert_eq!(rounds[1][1].row_hint, Some(1));

        assert_eq!(rounds[0].len(), 4);
        assert_eq!(rounds[0][0].seeds, [6, 11]); // 8 and 9 drops in
        assert_eq!(rounds[0][0].row_hint, Some(0));
        assert_eq!(rounds[0][1].seeds, [7, 10]); // 8 and 9 drops in
        assert_eq!(rounds[0][1].row_hint, Some(1));
        assert_eq!(rounds[0][2].seeds, [5, 12]); // 8 and 9 drops in
        assert_eq!(rounds[0][2].row_hint, Some(2));
        assert_eq!(rounds[0][3].seeds, [8, 9]); // 8 and 9 drops in
        assert_eq!(rounds[0][3].row_hint, Some(3));
    }
}
