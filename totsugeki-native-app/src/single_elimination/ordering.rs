use totsugeki::{matches::Match, player::Participants};

use crate::{convert, DisplayableMatch};

pub fn partition_winner_bracket(
    matches: Vec<Match>,
    participants: &Participants,
) -> Vec<Vec<DisplayableMatch>> {
    let n = participants.len();
    let Some(mut npo2) =  n.checked_next_power_of_two() else {
        panic!("MATH");
    };
    let matches: Vec<DisplayableMatch> = matches.iter().map(|m| convert(m, participants)).collect();
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

            let remaining_players = participants.len() - byes;
            let split = remaining_players / 2;
            // TODO use drain
            let tmp = remaining_matches.clone();
            let (first_round_matches, matches) = tmp.split_at(split);
            remaining_matches = matches.to_vec();
            partition.push(first_round_matches.to_vec());
            continue;
        } else {
            npo2 /= 2;
            let split = npo2 / 2;
            let (round, matches) = if remaining_matches.len() == 1 {
                // NOTE: I really don't like the unwrap but assigning
                // `remaining_matches` to an empty vec produces a warning
                // TODO remove unwrap
                let tmp = remaining_matches.drain(0..1).next().unwrap();
                (vec![tmp], vec![])
            } else {
                let (a, b) = remaining_matches.split_at(split);
                (a.to_vec(), b.to_vec())
            };
            partition.push(round.to_vec());
            remaining_matches = matches.to_vec();
            continue;
        }
    }

    partition
}

/// Matches are given positionnal hints
pub fn reorder_first_round(rounds: &mut [Vec<DisplayableMatch>]) {
    assert!(rounds.len() >= 2);

    let mut round_1 = rounds[0].clone();
    let number_of_matches_in_round_1 = rounds[1].len() * 2;

    for (i, m) in rounds[1].iter().enumerate() {
        let row_hint_1 = 2 * i;
        let row_hint_2 = 2 * i + 1;

        let seed_1 = m.seeds[0];
        // (first) player of round 1 with highest seed is expected to win
        if let Some(m) = round_1.iter_mut().find(|r1_m| r1_m.seeds[0] == seed_1) {
            m.row_hint = Some(row_hint_1);
        }
        let seed_2 = m.seeds[1];
        if let Some(m) = round_1.iter_mut().find(|r1_m| r1_m.seeds[0] == seed_2) {
            m.row_hint = Some(row_hint_2);
        }
    }

    for _i in 0..number_of_matches_in_round_1 - rounds[0].len() {
        round_1.push(DisplayableMatch::default())
    }

    rounds[0] = round_1;
}

#[cfg(test)]
mod tests {
    use totsugeki::{
        matches::Match,
        player::{Participants, Player},
    };

    use super::partition_winner_bracket;

    fn get_matches_and_participant(n: usize) -> (Vec<Match>, Participants) {
        let mut matches = vec![];
        let mut players = vec![];
        for _ in 0..n {
            matches.push(Match::default());
        }
        for i in 1..=n {
            players.push(Player::new(format!("p{i}")));
        }
        let participants = Participants::try_from(players).expect("participants");
        (matches, participants)
    }

    #[test]
    fn split_winner_bracket_3_participants() {
        let (matches, participants) = get_matches_and_participant(3);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 1, "first round");
        assert_eq!(partition[1].len(), 1, "second round");
    }

    #[test]
    fn split_winner_bracket_4_participants() {
        let (matches, participants) = get_matches_and_participant(4);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 2, "first round, 1-4 + 2-3");
        assert_eq!(partition[1].len(), 1, "second round, 1-2");
    }

    #[test]
    fn split_winner_bracket_5_participants() {
        let (matches, participants) = get_matches_and_participant(5);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 1, "first round, 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_6_participants() {
        let (matches, participants) = get_matches_and_participant(6);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 2, "first round, 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_7_participants() {
        let (matches, participants) = get_matches_and_participant(7);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 3, "first round, 2-7 + 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_8_participants() {
        let (matches, participants) = get_matches_and_participant(8);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_9_participants() {
        let (matches, participants) = get_matches_and_participant(9);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 1, "first round, 8-9");
        assert_eq!(partition[1].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[2].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[3].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_10_participants() {
        let (matches, participants) = get_matches_and_participant(10);
        let partition = partition_winner_bracket(matches, &participants);

        assert_eq!(partition[0].len(), 2, "first round, 7-10 + 8-9");
        assert_eq!(partition[1].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[2].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[3].len(), 1, "third round, 1-2");
    }
}
