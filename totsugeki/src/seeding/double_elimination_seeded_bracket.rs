//! generate seeded matches for double elimination bracket

use crate::{matches::Match, player::Participants, seeding::Error};

/// Returns looser bracket for a double elimination tournament
///
/// It is similar to `get_balanced_round_matches_top_seed_favored` where you
/// generate matches for 2 iterations at a time to compute winners moving to
/// next loser round.
///
/// The loosers of winner round X (power of 2 except for round 1) drop into
/// lower bracket and are matched against the winners of the previous looser
/// bracket round (also a power of two).
/// When dropping from winner round 1, you may get a bye match if your seed is
/// high for this looser round. Highest seeds are then matched against lowest
/// seeds.
///
/// Note that it is not balanced in the sense that seed 2 will only get one
/// match in loosers while bottom seed dropping to loosers has the longest road
/// ahead of them. It is only balanced when computing one round of looser
/// because any player in a round has at most one less match to play.
///
/// # Errors
/// thrown when math overflow happens
pub fn get_looser_bracket_matches_top_seed_favored(
    participants: &Participants,
) -> Result<Vec<Match>, Error> {
    let mut remaining_loosers = participants.clone().get_players_list();
    // winner of winner bracket is the only player not playing in lower bracket
    remaining_loosers.reverse();
    remaining_loosers.pop();
    // compute the number of winner rounds
    let mut loosers_by_round = vec![];
    let mut total_waves = 0;
    let mut n = 0;
    while n < participants.len() - 1 {
        n += match 2usize.checked_pow(total_waves) {
            Some(c) => c,
            None => return Err(Error::MathOverflow),
        };
        total_waves += 1;
    }
    // compute a looser wave by taking a power of two number of participants
    for i in 0..total_waves {
        // take 2^i participants for this wave starting from the last possible wave
        let number_of_loosers_for_this_round = match usize::checked_pow(2, i) {
            Some(power_of_two) => power_of_two.min(remaining_loosers.len()),
            None => return Err(Error::MathOverflow),
        };
        let mut loosers_for_this_round = vec![];
        for _ in 0..number_of_loosers_for_this_round {
            loosers_for_this_round.push(remaining_loosers.pop().expect("looser"));
        }
        loosers_by_round.push(loosers_for_this_round);
    }
    // get waves of looser by order of arrival
    loosers_by_round.reverse();

    let mut matches = vec![];
    let mut incoming_wave = vec![];
    let mut initial_wave = true;
    // initial looser wave <= next wave
    let skip_initial_wave_match_generation = loosers_by_round[0].len() <= loosers_by_round[1].len();
    // generate looser bracket matches
    for loosers_for_this_round in loosers_by_round {
        // Unlike single elimination, you cannot assume that
        // `loosers_for_this_round` is a multiple of two. Then you may have to
        // send the participants of the initial loser round to the next round
        let mut winners_of_previous_round = incoming_wave.clone();
        // sort incoming wave of looser by seed
        incoming_wave = vec![];
        incoming_wave.append(&mut loosers_for_this_round.clone());
        incoming_wave.append(&mut winners_of_previous_round);

        // Initial wave need to be thinned out when next wave has strictly more
        // people in it.
        // Example: in 10 man bracket, 2 people drop from winner, then 4 drop.
        if initial_wave && skip_initial_wave_match_generation {
            initial_wave = false;
            continue;
        }

        // generate matches for players without byes
        let byes = match (incoming_wave.len()).checked_next_power_of_two() {
            Some(next_higher_power_of_two) => next_higher_power_of_two - incoming_wave.len(),
            None => return Err(Error::MathOverflow),
        };
        let (p_with_bye, p_without_bye) = incoming_wave.split_at(byes);
        let half = p_without_bye.len() / 2;
        let (expected_winners, expected_loosers) = p_without_bye.split_at(half);
        let mut other_opponents = expected_loosers.to_vec();
        other_opponents.reverse();
        for (o1, o2) in expected_winners.iter().zip(other_opponents.iter()) {
            let seed_o1 = participants.get_seed(o1).expect("opponent 1 without bye");
            let seed_o2 = participants.get_seed(o2).expect("opponent 2 without bye");
            let m = Match::new_looser_bracket_match([seed_o1, seed_o2]);
            matches.push(m);
        }

        // take winners of this round and match them against players with byes
        let at_least_half =
            p_without_bye.len() / 2 + if p_without_bye.len() % 2 == 0 { 0 } else { 1 };
        let (winners_of_p_without_bye, _) = p_without_bye.split_at(at_least_half);
        let remaining = [p_with_bye, winners_of_p_without_bye].concat().clone();
        // with one exception: don't thin out more players during initial
        // looser round because there are always enough player from initial
        // wave after one round of match generation
        if initial_wave {
            initial_wave = false;
            incoming_wave = vec![];
            incoming_wave.append(&mut remaining.clone());
            continue;
        }
        let at_least_half = remaining.len() / 2 + if remaining.len() % 2 == 0 { 0 } else { 1 };
        let (expected_winners, expected_loosers) = remaining.split_at(at_least_half);
        let mut other_opponents = expected_loosers.to_vec();
        other_opponents.reverse();
        for (o1, o2) in expected_winners.iter().zip(other_opponents.iter()) {
            let seed_o1 = participants.get_seed(o1).expect("opponent 1 with bye");
            let seed_o2 = participants.get_seed(o2).expect("opponent 2 with bye");
            let m = Match::new_looser_bracket_match([seed_o1, seed_o2]);
            matches.push(m);
        }

        incoming_wave = vec![];
        incoming_wave.append(&mut expected_winners.to_vec());
    }

    // use unused remaining participants
    if !incoming_wave.is_empty() {
        let half = incoming_wave.len() / 2;
        let (expected_winners, expected_loosers) = incoming_wave.split_at(half);
        let mut expected_loosers = expected_loosers.to_vec();
        expected_loosers.reverse();
        for (o1, o2) in expected_winners.iter().zip(expected_loosers.iter()) {
            let seed_o1 = participants.get_seed(o1).expect("opponent 1 without bye");
            let seed_o2 = participants.get_seed(o2).expect("opponent 2 without bye");
            let m = Match::new_looser_bracket_match([seed_o1, seed_o2]);
            matches.push(m);
        }

        if expected_winners.len() == 2 {
            matches.push(Match::new_looser_bracket_match([2, 3]));
        }
    }

    Ok(matches)
}

#[cfg(test)]
mod tests {
    use crate::format::Format;
    use crate::matches::{Id as MatchId, Match, MatchGET};
    use crate::opponent::Opponent;
    use crate::player::{Id as PlayerId, Participants, Player};
    use crate::seeding::double_elimination_seeded_bracket::get_looser_bracket_matches_top_seed_favored;

    #[test]
    fn double_elimination_bracket_all_matches_generation_3_man() {
        // test if grand finals, grand finals reset and winner bracket is
        // generated
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names: Vec<String> = vec!["p1".into(), "p2".into(), "p3".into()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");

        let matches = Format::DoubleElimination
            .get_matches(&players)
            .expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches,
            vec![
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("id"),
                    [Opponent::Player(p2_id), Opponent::Player(p3_id)],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)],
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("id"),
                    [Opponent::Player(p1_id), Opponent::Unknown],
                    [1, 2],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)],
                ))
                .expect("match"),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [1, 2]),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [1, 2]),
            ],
            "returned {} matches with expected count of 5",
            matches.len()
        );
    }

    #[test]
    fn double_elimination_match_generation_4_man() {
        let mut participants = Participants::default();
        for _ in 0..4 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(matches.len(), 2, "expected 2 matches, got: {matches:?}");
        assert_eq!(
            matches,
            vec![
                Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
            ],
            "returned {} matches with expected count of 2",
            matches.len()
        );
    }

    #[test]
    fn double_elimination_match_generation_5_man() {
        let mut participants = Participants::default();
        for _ in 0..5 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            3,
            "matches seeding: {:?}\n got: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
        assert_eq!(
            matches,
            vec![
                Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
            ],
        );
    }

    #[test]
    fn double_elimination_match_generation_6_man() {
        let mut participants = Participants::default();
        for _ in 0..6 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            4,
            "\nmatches seeding: {:?}\n got: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "matches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }

    #[test]
    fn double_elimination_match_generation_7_man() {
        let mut participants = Participants::default();
        for _ in 0..7 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            5,
            "\nmatches seeding: {:?}\ngot: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 7]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "\nmatches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }

    #[test]
    fn double_elimination_match_generation_8_man() {
        let mut participants = Participants::default();
        for _ in 0..8 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            6,
            "\nmatches seeding: {:?}\ngot: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 8]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 7]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "matches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }

    #[test]
    fn double_elimination_match_generation_9_man() {
        let mut participants = Participants::default();
        for _ in 0..9 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            7,
            "\nmatches seeding: {:?}\ngot: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [8, 9]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 8]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 7]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "matches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }

    #[test]
    fn double_elimination_match_generation_10_man() {
        let mut participants = Participants::default();
        for _ in 0..10 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            8,
            "\nmatches seeding: {:?}\ngot: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [7, 10]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [8, 9]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 8]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 7]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "matches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }

    #[test]
    fn double_elimination_match_generation_11_man() {
        let mut participants = Participants::default();
        for _ in 0..11 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            9,
            "\nmatches seeding: {:?}\ngot: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 11]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [7, 10]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [8, 9]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 8]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 7]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "matches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }

    #[test]
    fn double_elimination_match_generation_12_man() {
        let mut participants = Participants::default();
        for _ in 0..12 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            10,
            "\nmatches seeding: {:?}\ngot: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 12]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 11]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [7, 10]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [8, 9]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 8]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 7]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "matches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }

    #[test]
    fn double_elimination_match_generation_16_man() {
        let mut participants = Participants::default();
        for _ in 0..16 {
            participants = participants
                .add_participant(Player::new("".into()))
                .expect("participant");
        }

        let matches = get_looser_bracket_matches_top_seed_favored(&participants).expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches.len(),
            14,
            "\nmatches seeding: {:?}\ngot: {matches:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
        );
        let expected_matches = vec![
            Match::looser_bracket_match(match_ids.pop().expect("id"), [9, 16]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [10, 15]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [11, 14]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [12, 13]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 12]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 11]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [7, 10]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [8, 9]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [5, 8]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [6, 7]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 6]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [4, 5]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [3, 4]),
            Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
        ];

        assert_eq!(
            matches,
            expected_matches,
            "matches seeding:\ngot     :{:?}\nexpected:{:?}",
            matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>(),
            expected_matches
                .iter()
                .map(crate::matches::Match::get_seeds)
                .collect::<Vec<[usize; 2]>>()
        );
    }
}
