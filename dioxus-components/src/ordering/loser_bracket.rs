//! Give positionnal hints to loser bracket matches

use crate::MinimalMatch;

/// Give positionnal hints to loser bracket matches
pub fn reorder(rounds: &mut [Vec<MinimalMatch>]) {
    // implementation is the same as winner bracket implementation. It seems
    // to also work for a loser bracket
    if rounds.len() < 2 {
        return;
    }

    let lb_rounds_count = rounds.len();

    // give row hints to last 3 rounds
    if lb_rounds_count > 2 {
        rounds[lb_rounds_count - 3][0].row_hint = Some(0);
        if rounds[lb_rounds_count - 3].len() > 1 {
            rounds[lb_rounds_count - 3][1].row_hint = Some(1);
        }
    }

    if lb_rounds_count > 1 {
        rounds[lb_rounds_count - 2][0].row_hint = Some(0);
    }

    rounds[lb_rounds_count - 1][0].row_hint = Some(0);

    // give hints to all other rounds
    for i in (0..rounds.len() - 2).rev() {
        let mut round = rounds[i].clone();
        let number_of_matches_in_round = rounds[i + 1].len() * 2;

        // iterate over previous round and set positional hints
        for (j, m) in rounds[i + 1].iter().enumerate() {
            let winner_seed = m.seeds[0];
            // (first) player of round 1 with highest seed is expected to win
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == winner_seed) {
                m.row_hint = Some(rounds[i + 1][j].row_hint.expect("") * 2);
            }
            let loser_seed = m.seeds[1];
            if let Some(m) = round.iter_mut().find(|r_m| r_m.seeds[0] == loser_seed) {
                if (lb_rounds_count - i) % 2 == 0 {
                    m.row_hint = rounds[i + 1][j].row_hint;
                } else {
                    // 7-10 (1), 8-9 (3)
                    m.row_hint = Some(rounds[i + 1][j].row_hint.expect("") * 2 + 1);
                }
            }
        }

        if i == 0 {
            if rounds.len() % 2 == 0 {
                for _ in 0..rounds[i + 1].len() - rounds[i].len() {
                    round.push(MinimalMatch::default())
                }
            } else {
                for _ in 0..number_of_matches_in_round - rounds[i].len() {
                    round.push(MinimalMatch::default())
                }
            }
        }

        round.sort_by_key(|m| m.row_hint);
        rounds[i] = round;
    }
}

#[cfg(test)]
mod tests {
    use crate::MinimalMatch;

    use super::reorder;

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
