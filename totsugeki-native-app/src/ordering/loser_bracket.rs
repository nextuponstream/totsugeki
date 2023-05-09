//! Give positionnal hints to loser bracket matches

use crate::DisplayableMatch;

/// Give positionnal hints to loser bracket matches
pub fn reorder(lb_rounds: &mut [Vec<DisplayableMatch>]) {
    for r in lb_rounds {
        for (i, m) in r.iter_mut().enumerate() {
            m.row_hint = Some(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::DisplayableMatch;

    use super::reorder;

    #[test]
    fn _3_players() {
        let mut rounds = [vec![DisplayableMatch::new([2, 3])]];

        reorder(&mut rounds);
        assert_eq!(rounds[0][0].row_hint, Some(0));
    }

    #[test]
    fn _4_players() {
        let mut rounds = [
            vec![DisplayableMatch::new([3, 4])],
            vec![DisplayableMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        assert_eq!(rounds[0][0].row_hint, Some(0));
        assert_eq!(rounds[1][0].row_hint, Some(0));
    }

    #[test]
    fn _5_players() {
        let mut rounds = [
            vec![DisplayableMatch::new([4, 5])],
            vec![DisplayableMatch::new([3, 4])],
            vec![DisplayableMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        assert_eq!(rounds[0][0].row_hint, Some(1));
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[2][0].row_hint, Some(0));
    }

    #[test]
    fn _6_players() {
        let mut rounds = [
            vec![DisplayableMatch::new([3, 6])],
            vec![DisplayableMatch::new([4, 5])],
            vec![DisplayableMatch::new([3, 4])],
            vec![DisplayableMatch::new([2, 3])],
        ];

        reorder(&mut rounds);
        assert_eq!(rounds[0][0].row_hint, Some(0));
        assert_eq!(rounds[0][1].row_hint, Some(1));
        assert_eq!(rounds[1][0].row_hint, Some(0));
        assert_eq!(rounds[2][0].row_hint, Some(0));
    }
}
