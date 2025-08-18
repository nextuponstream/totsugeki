//! Winner bracket

use crate::bracket::seeding::Seeding;
use crate::matches::Match;
// FIXME create error enum and handle math errors

/// Partition list of matches into rounds for a winner bracket
pub(crate) fn winner_bracket(matches: Vec<Match>, seeding: &Seeding) -> Vec<Vec<Match>> {
    let n = seeding.len();
    let Some(mut npo2) = n.checked_next_power_of_two() else {
        panic!("MATH");
    };
    let byes = npo2 - n;
    let mut remaining_matches = matches;
    let mut partition = vec![];
    let mut is_first_round = true;
    while !remaining_matches.is_empty() {
        if is_first_round {
            is_first_round = false;
            // 5 players, 2^npo2 >= 5 -> npo2 = 3
            // byes = npo2 - 5 = 3
            // 2³ = 8
            // 3 players dont play
            // 2 players have to play
            // 1 match
            // next round
            // 4 players => 2 matches
            // 2 players => 1 match

            // 4 players, 2^npo2 >= 4 -> npo2 = 2
            // byes = npo2 - 4 = 0
            // 2² = 4
            // 2 players don't play
            // no
            // 4 players, 2^byes == #participants -> byes = 0
            // 4 players, 2 matches
            // 2 players, 1 match

            // 3 players, 2^byes > #participants (3) -> npo2 = 2
            // byes = 4 - 3 = 1
            // 1 players does not play
            // 2 players play
            // 1 match
            // 2 players play
            // 1 match

            let remaining_players = seeding.len() - byes;
            let split = remaining_players / 2;
            // TODO use drain
            let tmp = remaining_matches.clone();
            let (first_round_matches, matches) = tmp.split_at(split);
            remaining_matches = matches.to_vec();
            partition.push(first_round_matches.to_vec());
        } else {
            npo2 /= 2;
            let split = npo2 / 2;
            let (round, matches) = if remaining_matches.len() == 1 {
                // NOTE: I really don't like the unwrap but assigning
                // `remaining_matches` to an empty vec produces a warning
                // TODO remove unwrap
                let tmp = remaining_matches
                    .drain(0..1)
                    .next()
                    .expect("iterated over last match");
                (vec![tmp], vec![])
            } else {
                let (a, b) = remaining_matches.split_at(split);
                (a.to_vec(), b.to_vec())
            };
            partition.push(round.clone());
            remaining_matches.clone_from(&matches);
        }
    }

    partition
}

#[cfg(test)]
mod tests {
    use super::winner_bracket;
    use crate::bracket::seeding::Seeding;
    use crate::matches::Match;
    use crate::ID;

    fn get_matches_and_seeding(n: usize) -> (Vec<Match>, Seeding) {
        let mut matches = vec![];
        let mut seeding = vec![];
        for _ in 0..n {
            matches.push(Match::default());
        }
        for _i in 1..=n {
            seeding.push(ID::new_v4());
        }
        (matches, Seeding::new(seeding).unwrap())
    }

    #[test]
    fn split_winner_bracket_3_participants() {
        let (matches, participants) = get_matches_and_seeding(3);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 1, "first round");
        assert_eq!(partition[1].len(), 1, "second round");
    }

    #[test]
    fn split_winner_bracket_4_participants() {
        let (matches, participants) = get_matches_and_seeding(4);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 2, "first round, 1-4 + 2-3");
        assert_eq!(partition[1].len(), 1, "second round, 1-2");
    }

    #[test]
    fn split_winner_bracket_5_participants() {
        let (matches, participants) = get_matches_and_seeding(5);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 1, "first round, 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_6_participants() {
        let (matches, participants) = get_matches_and_seeding(6);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 2, "first round, 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_7_participants() {
        let (matches, participants) = get_matches_and_seeding(7);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 3, "first round, 2-7 + 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_8_participants() {
        let (matches, participants) = get_matches_and_seeding(8);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_9_participants() {
        let (matches, participants) = get_matches_and_seeding(9);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 1, "first round, 8-9");
        assert_eq!(partition[1].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[2].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[3].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_10_participants() {
        let (matches, participants) = get_matches_and_seeding(10);
        let partition = winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 2, "first round, 7-10 + 8-9");
        assert_eq!(partition[1].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[2].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[3].len(), 1, "third round, 1-2");
    }
}
