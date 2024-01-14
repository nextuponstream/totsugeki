//! generate seeded matches for double elimination bracket

use std::ops::ControlFlow;

use crate::{matches::Match, player::Id as PlayerId, seeding::Error};

/// Get seed of player from seeding
fn get_seed_of(player: &PlayerId, seeding: &[PlayerId]) -> usize {
    assert!(seeding.contains(player));
    for (i, p) in seeding.iter().enumerate() {
        if p == player {
            return i + 1;
        }
    }
    unreachable!("player somehow is not in the seeding")
}

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
pub fn get_loser_bracket_matches_top_seed_favored(
    seeding: &[PlayerId],
) -> Result<Vec<Match>, Error> {
    let losers_by_round = match partition_players_of_loser_bracket(seeding) {
        Ok(value) => value,
        Err(value) => return value,
    };

    let mut matches = vec![];
    let mut incoming_players_of_this_wave = vec![];
    let mut initial_wave = true;
    // initial looser wave <= next wave
    let skip_initial_wave_match_generation = losers_by_round[0].len() <= losers_by_round[1].len();
    // generate loser bracket matches
    for losers_for_this_round in losers_by_round {
        if let ControlFlow::Break(()) = fill_incoming_wave(
            &mut incoming_players_of_this_wave,
            &losers_for_this_round,
            &mut initial_wave,
            skip_initial_wave_match_generation,
        ) {
            continue;
        }

        let tmp = incoming_players_of_this_wave.clone();
        let wave = form_wave(&tmp)?;
        let (p_with_bye, p_without_bye) =
            generate_matches_of_first_round_in_wave(wave, seeding, &mut matches);

        let Some(remaining) = fun_name(
            p_without_bye,
            p_with_bye,
            &mut initial_wave,
            &mut incoming_players_of_this_wave,
        ) else {
            continue;
        };
        // reason: readability
        #[allow(clippy::bool_to_int_with_if)]
        let at_least_half = remaining.len() / 2 + if remaining.len() % 2 == 0 { 0 } else { 1 };
        let (expected_winners, expected_losers) = remaining.split_at(at_least_half);
        let mut other_opponents = expected_losers.to_vec();
        other_opponents.reverse();
        for (o1, o2) in expected_winners.iter().zip(other_opponents.iter()) {
            let seed_o1 = get_seed_of(o1, seeding);
            let seed_o2 = get_seed_of(o2, seeding);
            let m = Match::new_looser_bracket_match([seed_o1, seed_o2]);
            matches.push(m);
        }

        incoming_players_of_this_wave = vec![];
        incoming_players_of_this_wave.append(&mut expected_winners.to_vec());
    }

    // use unused remaining participants
    if !incoming_players_of_this_wave.is_empty() {
        let half = incoming_players_of_this_wave.len() / 2;
        let (expected_winners, expected_loosers) = incoming_players_of_this_wave.split_at(half);
        let mut expected_loosers = expected_loosers.to_vec();
        expected_loosers.reverse();
        for (o1, o2) in expected_winners.iter().zip(expected_loosers.iter()) {
            let seed_o1 = get_seed_of(o1, seeding);
            let seed_o2 = get_seed_of(o2, seeding);
            let m = Match::new_looser_bracket_match([seed_o1, seed_o2]);
            matches.push(m);
        }

        if expected_winners.len() == 2 {
            matches.push(Match::new_looser_bracket_match([2, 3]));
        }
    }

    Ok(matches)
}

/// qiej
fn fun_name(
    p_without_bye: &[uuid::Uuid],
    p_with_bye: &[uuid::Uuid],
    initial_wave: &mut bool,
    incoming_players_of_this_wave: &mut Vec<uuid::Uuid>,
) -> Option<Vec<uuid::Uuid>> {
    #[allow(clippy::bool_to_int_with_if)]
    let at_least_half = p_without_bye.len() / 2 + if p_without_bye.len() % 2 == 0 { 0 } else { 1 };
    let (winners_of_p_without_bye, _) = p_without_bye.split_at(at_least_half);
    let mut remaining = [p_with_bye, winners_of_p_without_bye].concat();
    if *initial_wave {
        *initial_wave = false;
        *incoming_players_of_this_wave = vec![];
        incoming_players_of_this_wave.append(&mut remaining);
        return None;
    }
    Some(remaining)
}

/// Generate the first out of the two round (of matches) in a wave, using the
/// players without byes. Each match has one of the expected winners and one of
/// the expected loser
fn generate_matches_of_first_round_in_wave<'a>(
    wave: Wave<'a>,
    seeding: &'a [uuid::Uuid],
    matches: &'a mut Vec<Match>,
) -> (&'a [uuid::Uuid], &'a [uuid::Uuid]) {
    let p_with_bye = wave.players_with_bye;
    let p_without_bye = wave.players_without_bye;
    let expected_winners = wave.expected_winners;
    let expected_losers = wave.expected_losers;
    for (o1, o2) in expected_winners.iter().zip(expected_losers.iter()) {
        let seed_o1 = get_seed_of(o1, seeding);
        let seed_o2 = get_seed_of(o2, seeding);
        let m = Match::new_looser_bracket_match([seed_o1, seed_o2]);
        matches.push(m);
    }
    (p_with_bye, p_without_bye)
}

/// Players of a loser bracket wave. A wave is two consecutive rounds of the
/// loser bracket.
///
/// In the first round of a wave, players without byes fight
/// each other to determine who will move on to the next round, where they will
/// fight the players with bye. The players that move on to the second round
/// are the
struct Wave<'a> {
    /// oqiwje
    players_with_bye: &'a [uuid::Uuid],
    /// owqijeh
    players_without_bye: &'a [uuid::Uuid],
    /// players that are expected to move on in the matches between players
    /// without byes for this wave
    expected_winners: &'a [uuid::Uuid],
    /// players that are not expected to move on in the matches between players
    /// without byes for this wave
    expected_losers: Vec<uuid::Uuid>,
}

/// Returns wave of players. See `Wave` documentation for more information
fn form_wave(incoming_players_of_wave: &Vec<uuid::Uuid>) -> Result<Wave, Error> {
    let byes = match (incoming_players_of_wave.len()).checked_next_power_of_two() {
        Some(next_higher_power_of_two) => next_higher_power_of_two - incoming_players_of_wave.len(),
        None => return Err(Error::MathOverflow),
    };
    let (players_with_bye, players_without_bye) = incoming_players_of_wave.split_at(byes);
    let half = players_without_bye.len() / 2;
    let (expected_winners, expected_losers) = players_without_bye.split_at(half);
    let mut expected_losers = expected_losers.to_vec();
    expected_losers.reverse();
    Ok(Wave {
        players_with_bye,
        players_without_bye,
        expected_winners,
        expected_losers,
    })
}

/// Returns `ControlFlow::Break` when the players from the initial wave should
/// be group together with the player for the next wave
fn fill_incoming_wave(
    incoming_wave: &mut Vec<uuid::Uuid>,
    losers_for_this_round: &[uuid::Uuid],
    initial_wave: &mut bool,
    skip_initial_wave_match_generation: bool,
) -> ControlFlow<()> {
    let mut winners_of_previous_round = incoming_wave.clone();
    *incoming_wave = vec![];
    incoming_wave.append(&mut losers_for_this_round.to_vec());
    incoming_wave.append(&mut winners_of_previous_round);
    if *initial_wave && skip_initial_wave_match_generation {
        *initial_wave = false;
        return ControlFlow::Break(());
    }
    ControlFlow::Continue(())
}

/// Partitions players by "waves". Waves are made of the winner of the previous
/// loser bracket round and the incoming player from the winner bracket (who
/// lost a mathc)
fn partition_players_of_loser_bracket(
    seeding: &[uuid::Uuid],
) -> Result<Vec<Vec<uuid::Uuid>>, Result<Vec<Match>, Error>> {
    let mut remaining_loosers = seeding.to_vec();
    remaining_loosers.reverse();
    remaining_loosers.pop();
    let mut losers_by_round = vec![];
    let mut total_waves = 0;
    let mut n = 0;
    while n < seeding.len() - 1 {
        n += match 2usize.checked_pow(total_waves) {
            Some(c) => c,
            None => return Err(Err(Error::MathOverflow)),
        };
        total_waves += 1;
    }
    for i in 0..total_waves {
        // take 2^i participants for this wave starting from the last possible wave
        let number_of_losers_for_this_round = match usize::checked_pow(2, i) {
            Some(power_of_two) => power_of_two.min(remaining_loosers.len()),
            None => return Err(Err(Error::MathOverflow)),
        };
        let mut loosers_for_this_round = vec![];
        for _ in 0..number_of_losers_for_this_round {
            loosers_for_this_round.push(remaining_loosers.pop().expect("looser"));
        }
        losers_by_round.push(loosers_for_this_round);
    }
    losers_by_round.reverse();
    Ok(losers_by_round)
}

#[cfg(test)]
mod tests {
    use crate::format::Format;
    use crate::matches::{Id as MatchId, Match};
    use crate::opponent::Opponent;
    use crate::player::{Participants, Player};
    use crate::seeding::double_elimination_seeded_bracket::get_loser_bracket_matches_top_seed_favored;

    #[test]
    fn matches_generation_3_man() {
        // test if grand finals, grand finals reset and winner bracket is
        // generated
        let mut players = vec![];
        for i in 1..=3 {
            let p = Player::new(format!("p{i}"));
            players.push(p);
        }
        let participants = Participants::try_from(players.clone()).expect("participants");
        players.reverse();
        players.push(Player::new("don't use".into()));
        players.reverse();

        let matches = Format::DoubleElimination
            .generate_matches(&participants.get_seeding())
            .expect("matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        assert_eq!(
            matches,
            vec![
                Match {
                    id: match_ids.pop().expect("id"),
                    players: [
                        Opponent::Player(players[2].get_id()),
                        Opponent::Player(players[3].get_id())
                    ],
                    seeds: [2, 3],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)],
                },
                Match {
                    id: match_ids.pop().expect("id"),
                    players: [Opponent::Player(players[1].get_id()), Opponent::Unknown],
                    seeds: [1, 2],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)],
                },
                Match::looser_bracket_match(match_ids.pop().expect("id"), [2, 3]),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [1, 2]),
                Match::looser_bracket_match(match_ids.pop().expect("id"), [1, 2]),
            ],
            "returned {} matches with expected count of 5",
            matches.len()
        );
    }

    #[test]
    fn match_generation_4_man() {
        let mut participants = Participants::default();
        let mut seeding = vec![];
        for _ in 0..4 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
        let mut match_ids: Vec<MatchId> = matches.iter().map(Match::get_id).rev().collect();
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
    fn match_generation_5_man() {
        let mut participants = Participants::default();
        let mut seeding = vec![];
        for _ in 0..5 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
    fn match_generation_6_man() {
        let mut participants = Participants::default();
        let mut seeding = vec![];
        for _ in 0..6 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
    fn match_generation_7_man() {
        let mut participants = Participants::default();
        let mut seeding = vec![];
        for _ in 0..7 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
    fn match_generation_8_man() {
        let mut participants = Participants::default();
        let mut seeding = vec![];
        for _ in 0..8 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
    fn match_generation_9_man() {
        let mut participants = Participants::default();
        let mut seeding = vec![];
        for _ in 0..9 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
    fn match_generation_10_man() {
        let mut participants = Participants::default();
        let mut seeding = vec![];
        for _ in 0..10 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
        let mut seeding = vec![];
        for _ in 0..11 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
        let mut seeding = vec![];
        for _ in 0..12 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
        let mut seeding = vec![];
        for _ in 0..16 {
            let player = Player::new(String::new());
            seeding.push(player.get_id());
            participants = participants.add_participant(player).expect("participant");
        }

        let matches = get_loser_bracket_matches_top_seed_favored(&seeding).expect("matches");
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
