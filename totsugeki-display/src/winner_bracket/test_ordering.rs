//! Test ordering of matches in winner bracket

#[cfg(test)]
mod tests {
    use crate::winner_bracket::reorder;
    use crate::MinimalMatch;

    #[test]
    fn _3_players() {
        let mut rounds = [
            vec![MinimalMatch::new([2, 3])],
            vec![MinimalMatch::new([1, 2])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds[0][0].row_hint, Some(1));
        assert_eq!(rounds[1][0].row_hint, Some(0));
    }

    #[test]
    fn _4_players() {
        let mut rounds = [
            vec![MinimalMatch::new([1, 4]), MinimalMatch::new([2, 3])],
            vec![MinimalMatch::new([1, 2])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds[0][0].row_hint, Some(0));
        assert_eq!(rounds[0][1].row_hint, Some(1));

        assert_eq!(rounds[1][0].row_hint, Some(0));
    }

    #[test]
    fn _5_players() {
        let mut rounds = [
            vec![MinimalMatch::new([4, 5])],
            vec![MinimalMatch::new([1, 4]), MinimalMatch::new([2, 3])],
            vec![MinimalMatch::new([1, 2])],
        ];

        reorder(&mut rounds);

        // 3 filler match (0-2), then first real match
        assert_eq!(rounds[0][3].row_hint, Some(1));

        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[1][1].row_hint, Some(1));

        assert_eq!(rounds[2][0].row_hint, Some(0));
    }

    #[test]
    fn _9_players() {
        let mut rounds = [
            vec![MinimalMatch::new([8, 9])],
            vec![
                MinimalMatch::new([1, 8]),
                MinimalMatch::new([2, 7]),
                MinimalMatch::new([3, 6]),
                MinimalMatch::new([4, 5]),
            ],
            vec![MinimalMatch::new([1, 4]), MinimalMatch::new([2, 3])],
            vec![MinimalMatch::new([1, 2])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds[0][7].row_hint, Some(1)); // 8-9

        assert_eq!(rounds[1][0].row_hint, Some(0)); // 1-8
        assert_eq!(rounds[1][1].row_hint, Some(1)); // 4-5
        assert_eq!(rounds[1][2].row_hint, Some(2)); // 2-7
        assert_eq!(rounds[1][3].row_hint, Some(3)); // 3-6

        assert_eq!(rounds[2][0].row_hint, Some(0)); // 1-4
        assert_eq!(rounds[2][1].row_hint, Some(1)); // 2-3

        assert_eq!(rounds[3][0].row_hint, Some(0)); // 1-2
    }
}
