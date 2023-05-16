//! Give positionnal hints to winner bracket matches
use crate::DisplayableMatch;

/// Give positionnal hints to winner bracket matches
pub fn reorder(rounds: &mut [Vec<DisplayableMatch>]) {
    if rounds.len() < 2 {
        return;
    }

    // set hint for all rounds except last two
    // traverse from last to first
    for i in (0..rounds.len() - 2).rev() {
        let mut round = rounds[i].clone();
        let number_of_matches_in_round = rounds[i + 1].len() * 2;

        // iterate over previous round and set positional hints
        for (j, m) in rounds[i + 1].iter().enumerate() {
            let row_hint_1 = 2 * j;
            let row_hint_2 = 2 * j + 1;

            let seed_1 = m.seeds[0];
            // (first) player of round 1 with highest seed is expected to win
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == seed_1) {
                m.row_hint = Some(row_hint_1);
            }
            let seed_2 = m.seeds[1];
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == seed_2) {
                m.row_hint = Some(row_hint_2);
            }
        }

        if i == 0 {
            for _ in 0..number_of_matches_in_round - rounds[i].len() {
                round.push(DisplayableMatch::default())
            }
        }

        // sort row i+1 so unsorted row i can be sorted next iteration
        // NOTE: for round 1, filler matches are first after sorting
        round.sort_by_key(|m| m.row_hint);
        rounds[i] = round;
    }

    // round before last round
    rounds[rounds.len() - 2][0].row_hint = Some(0);
    // when there is exactly 3 players
    if rounds[rounds.len() - 2].len() == 1 {
        rounds[rounds.len() - 2][0].row_hint = Some(1);
    } else {
        rounds[rounds.len() - 2][1].row_hint = Some(1);
    }

    // last round
    rounds[rounds.len() - 1][0].row_hint = Some(0);
}

#[cfg(test)]
mod tests {
    use super::reorder;
    use crate::DisplayableMatch;

    #[test]
    fn _3_players() {
        let mut rounds = [
            vec![DisplayableMatch::new([2, 3])],
            vec![DisplayableMatch::new([1, 2])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds[0][0].row_hint, Some(1));
        assert_eq!(rounds[1][0].row_hint, Some(0));
    }

    #[test]
    fn _4_players() {
        let mut rounds = [
            vec![DisplayableMatch::new([1, 4]), DisplayableMatch::new([2, 3])],
            vec![DisplayableMatch::new([1, 2])],
        ];

        reorder(&mut rounds);

        assert_eq!(rounds[0][0].row_hint, Some(0));
        assert_eq!(rounds[0][1].row_hint, Some(1));

        assert_eq!(rounds[1][0].row_hint, Some(0));
    }

    #[test]
    fn _5_players() {
        let mut rounds = [
            vec![DisplayableMatch::new([4, 5])],
            vec![DisplayableMatch::new([1, 4]), DisplayableMatch::new([2, 3])],
            vec![DisplayableMatch::new([1, 2])],
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
            vec![DisplayableMatch::new([8, 9])],
            vec![
                DisplayableMatch::new([1, 8]),
                DisplayableMatch::new([2, 7]),
                DisplayableMatch::new([3, 6]),
                DisplayableMatch::new([4, 5]),
            ],
            vec![DisplayableMatch::new([1, 4]), DisplayableMatch::new([2, 3])],
            vec![DisplayableMatch::new([1, 2])],
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
