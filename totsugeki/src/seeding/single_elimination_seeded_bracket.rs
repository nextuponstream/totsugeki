//! Generate seeded matches for single elimination

use crate::{
    matches::Match,
    opponent::Opponent,
    player::{Id as PlayerId, Participants, Player},
    seeding::Error,
};

use super::seeding_initial_round;

/// Returns tournament matches for `n` players in a list. Used for generating
/// single elimination bracket or winner bracket in double elimination format.
///
/// Top seed plays the least matches. They will face predicted higher seeds
/// only later in the bracket. Top seed plays at most one more match than
/// anyone else.
///
/// # Errors
/// Throws error when math overflow happens
pub fn get_balanced_round_matches_top_seed_favored(
    players: &Participants,
) -> Result<Vec<Match>, Error> {
    // Matches are built bottom-up:
    // * for n
    // * compute #byes = `next_power_of_two(n)` - n
    // * for round 1, assign the #byes top seeds their bye match
    //   NOTE: you don't need to add those matches to the list of generated matches
    // * for round 1, find top+low seed, assign them a match and repeat until no players are left
    // * for round 2, select next 4 matches
    // * ...
    let n = players.len();
    let byes = match n.checked_next_power_of_two() {
        Some(b) => b - n,
        None => return Err(Error::MathOverflow),
    };
    let mut remaining_byes = byes;
    let mut this_round: Vec<Match> = vec![];
    let mut round_matches: Vec<Vec<Match>> = vec![];

    // Initialize bye matches in first round
    let mut available_players: Vec<usize> = (1..=n).collect();
    (0..byes).for_each(|_| {
        let _top_seed = available_players.remove(0);
    });

    let first_round = match n.checked_next_power_of_two() {
        Some(i) => i,
        None => return Err(Error::MathOverflow),
    };
    let second_round = first_round / 2;
    let mut i = first_round;
    while i > 1 {
        while !available_players.is_empty() {
            if first_round == i {
                seeding_initial_round(&mut available_players, players, &mut this_round);
            } else if second_round == i {
                let top_seed = available_players.remove(0);
                let top_seed_player = *players
                    .clone()
                    .get_players_list()
                    .iter()
                    .map(Player::get_id)
                    .collect::<Vec<PlayerId>>()
                    .get(top_seed - 1)
                    .expect("player id");
                let bottom_seed = available_players.pop().expect("bottom seed");
                let bottom_seed_player = *players
                    .clone()
                    .get_players_list()
                    .iter()
                    .map(Player::get_id)
                    .collect::<Vec<PlayerId>>()
                    .get(bottom_seed - 1)
                    .expect("player id");
                let player_1 = if remaining_byes > 0 && top_seed <= byes {
                    remaining_byes -= 1;
                    Opponent::Player(top_seed_player)
                } else {
                    Opponent::Unknown
                };
                let player_2 = if remaining_byes > 0 && bottom_seed <= byes {
                    remaining_byes -= 1;
                    Opponent::Player(bottom_seed_player)
                } else {
                    Opponent::Unknown
                };

                this_round.push(
                    Match::new([player_1, player_2], [top_seed, bottom_seed]).expect("match"),
                );
            } else {
                let top_seed = available_players.remove(0);
                let bottom_seed = available_players.pop().expect("bottom seed");

                this_round.push(
                    Match::new(
                        [Opponent::Unknown, Opponent::Unknown],
                        [top_seed, bottom_seed],
                    )
                    .expect("match"),
                );
            }
        }

        // empty iteration variable `this_round` into round_matches
        round_matches.push(this_round.drain(..).collect());
        i /= 2;
        available_players = (1..=i).collect();
    }

    Ok(round_matches.into_iter().flatten().collect())
}
#[cfg(test)]
mod tests {
    use crate::matches::{Id as MatchId, MatchGET};
    use crate::seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored;
    use crate::seeding::{seed, Method};
    use crate::{
        matches::Match,
        opponent::Opponent,
        player::{Participants, Player},
    };
    use rand::Rng;

    #[test]
    fn single_elimination_favors_top_seed_3_man() {
        let n = 3;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego").get_id();
        let pink = players.get(1).expect("pink").get_id();
        let cute_cat = players.get(2).expect("cute_cat").get_id();

        let players = Participants::try_from(players_copy).expect("players");
        let matches =
            get_balanced_round_matches_top_seed_favored(&players).expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink), Opponent::Player(cute_cat)],
                [2, 3],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(diego), Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    fn single_elimination_favors_top_seed_4_man() {
        let n = 4;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego").get_id();
        let pink = players.get(1).expect("pink").get_id();
        let guy = players.get(2).expect("guy").get_id();
        let cute_cat = players.get(3).expect("cute_cat").get_id();

        let players = Participants::try_from(players_copy).expect("players");
        let matches =
            get_balanced_round_matches_top_seed_favored(&players).expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(diego), Opponent::Player(cute_cat)],
                [1, 4],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink), Opponent::Player(guy)],
                [2, 3],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    fn single_elimination_favors_top_seed_5_man() {
        let n = 5;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego").get_id();
        let pink = players.get(1).expect("pink").get_id();
        let average_player = players.get(2).expect("pink").get_id();
        let guy = players.get(3).expect("guy").get_id();
        let cute_cat = players.get(4).expect("cute_cat").get_id();

        let players = Participants::try_from(players_copy).expect("players");
        let matches =
            get_balanced_round_matches_top_seed_favored(&players).expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(guy), Opponent::Player(cute_cat)],
                [4, 5],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(diego), Opponent::Unknown],
                [1, 4],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink), Opponent::Player(average_player)],
                [2, 3],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    fn single_elimination_favors_top_seed_6_man() {
        let n = 6;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego").get_id();
        let pink = players.get(1).expect("pink").get_id();
        let pink_nemesis = players.get(2).expect("pink_nemesis").get_id();
        let average_player = players.get(3).expect("pink").get_id();
        let guy = players.get(4).expect("guy").get_id();
        let cute_cat = players.get(5).expect("cute_cat").get_id();

        let players = Participants::try_from(players_copy).expect("players");
        let matches =
            get_balanced_round_matches_top_seed_favored(&players).expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink_nemesis), Opponent::Player(cute_cat)],
                [3, 6],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(average_player), Opponent::Player(guy)],
                [4, 5],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(diego), Opponent::Unknown],
                [1, 4],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink), Opponent::Unknown],
                [2, 3],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn single_elimination_favors_top_seed_7_man() {
        let n = 7;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego").get_id();
        let pink = players.get(1).expect("pink").get_id();
        let pink_nemesis = players.get(2).expect("pink_nemesis").get_id();
        let average_player = players.get(3).expect("pink").get_id();
        let guy = players.get(4).expect("guy").get_id();
        let fg_enjoyer = players.get(5).expect("fg_enjoyer").get_id();
        let cute_cat = players.get(6).expect("cute_cat").get_id();

        let players = Participants::try_from(players_copy).expect("players");
        let matches =
            get_balanced_round_matches_top_seed_favored(&players).expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink), Opponent::Player(cute_cat)],
                [2, 7],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink_nemesis), Opponent::Player(fg_enjoyer)],
                [3, 6],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(average_player), Opponent::Player(guy)],
                [4, 5],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(diego), Opponent::Unknown],
                [1, 4],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [2, 3],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn single_elimination_favors_top_seed_8_man() {
        let n = 8;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego").get_id();
        let pink = players.get(1).expect("pink").get_id();
        let pink_nemesis = players.get(2).expect("pink_nemesis").get_id();
        let big_body_enjoyer = players.get(3).expect("big_body_enjoyer").get_id();
        let average_player = players.get(4).expect("pink").get_id();
        let guy = players.get(5).expect("guy").get_id();
        let fg_enjoyer = players.get(6).expect("fg_enjoyer").get_id();
        let cute_cat = players.get(7).expect("cute_cat").get_id();

        let players = Participants::try_from(players_copy).expect("players");
        let matches =
            get_balanced_round_matches_top_seed_favored(&players).expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(diego), Opponent::Player(cute_cat)],
                [1, 8],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink), Opponent::Player(fg_enjoyer)],
                [2, 7],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Player(pink_nemesis), Opponent::Player(guy)],
                [3, 6],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [
                    Opponent::Player(big_body_enjoyer),
                    Opponent::Player(average_player),
                ],
                [4, 5],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 4],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [2, 3],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
            Match::try_from(MatchGET::new(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
                [(0, 0), (0, 0)],
            ))
            .expect("match"),
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    #[ignore]
    fn single_elimination_matches_generation_does_not_break_for_with_high_entrance_numbers() {
        (0..10).for_each(|_| {
            let mut rng = rand::thread_rng();
            let n = rng.gen_range(3..3000);
            let mut players: Vec<Player> = vec![];
            (0..n).for_each(|i| {
                players.push(Player::new(format!("player{i}")));
            });
            let players = Participants::try_from(players).expect("players");
            let players = seed(&Method::Strict, players.clone(), players).expect("seeded players");
            let _matches = get_balanced_round_matches_top_seed_favored(&players);
        });
    }
}
