//! Seed brackets with seeding methods

use crate::{
    matches::{Match, Opponent},
    player::{Error as PlayerError, Players},
};
use rand::prelude::*;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};

/// Seeding method
#[derive(Copy, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Method {
    /// Randomize who plays against who
    Random,
    /// Sort players by perceived strength to avoid pitting them against each
    /// other early in the bracket
    Strict,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Random => write!(f, "random"),
            Method::Strict => write!(f, "strict"),
        }
    }
}

/// Seeding method parsing error
#[derive(Debug)]
pub enum ParsingError {
    /// Unknown seeding method was found
    Unknown,
}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Unknown => writeln!(f, "Seeding method is unknown"),
        }
    }
}

impl std::str::FromStr for Method {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "random" => Self::Random,
            "strict" => Self::Strict,
            _ => return Err(ParsingError::Unknown),
        })
    }
}

impl Default for Method {
    fn default() -> Self {
        Self::Strict
    }
}

/// Error while seeding
#[derive(Debug, PartialEq)]
pub enum Error {
    /// You cannot seed a bracket of 0 players
    NotEnoughPlayers,
    /// The os generator panicked while generating a random number
    Rng,
    /// Shuffle did yield players
    Shuffle(PlayerError),
}

/// Returns ordered list of players according to the seeding method.
///
/// With `Strict` method, `players` are expected to be ranked from strongest to
/// weakest.
///
/// # Errors
/// Returns an error when filling an empty bracket
pub fn seed(method: &Method, players: Players) -> Result<Players, Error> {
    if players.len() < 3 {
        return Err(Error::NotEnoughPlayers);
    }

    match method {
        Method::Random => {
            let mut key = [0u8; 16];
            OsRng.try_fill_bytes(&mut key)?;
            let mut rng = OsRng::default();
            let mut players = players.get_players();
            players.shuffle(&mut rng);
            let players = Players::from(players)?;
            Ok(players)
        }
        Method::Strict => Ok(players),
    }
}

/// Matches from bracket, sorted by rounds
struct RoundMatches(Vec<Vec<Match>>);

impl std::fmt::Display for RoundMatches {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (i, r) in self.0.iter().enumerate() {
            writeln!(f, "Round: {}", i + 1)?;
            for m in r.iter() {
                writeln!(f, "  - match: {m}")?;
            }
        }
        Ok(())
    }
}

/// Returns tournament matches for `n` players. Matches are separated by rounds.
///
/// Top seed plays the least matches. He will face predicted higher seeds only
/// later in the bracket. Top seed plays at most one more match than anyone
/// else.
#[must_use]
pub fn get_balanced_round_matches_top_seed_favored(players: &Players) -> Vec<Vec<Match>> {
    // Matches are built bottom-up:
    // * for n
    // * compute #byes = `next_power_of_two(n)` - n
    // * for round 1, assign the #byes top seeds their bye match
    // * for round 1, find top+low seed, assign them a match and repeat until no players are left
    // * for round 2, select next 4 matches
    // * ...
    let n = players.len();
    let byes = n.next_power_of_two() - n;
    let mut this_round: Vec<Match> = vec![];
    let mut round_matches = vec![];

    // Initialize bye matches in first round
    let mut available_players: Vec<usize> = (1..=n).collect();
    (0..byes).for_each(|i| {
        let top_seed = available_players.remove(0);
        let top_seed_player = *players
            .clone()
            .get_players()
            .get(top_seed - 1)
            .expect("player id");
        let bottom_seed = n.next_power_of_two() - i;
        this_round.push(
            Match::new(
                [Opponent::Player(top_seed_player), Opponent::Bye],
                [top_seed, bottom_seed],
            )
            .expect("match"),
        );
    });

    let mut i = n.next_power_of_two();
    let mut initial_round = true;
    while i > 1 {
        while !available_players.is_empty() {
            if initial_round {
                let top_seed = available_players.remove(0);
                let top_seed_player = *players
                    .clone()
                    .get_players()
                    .get(top_seed - 1)
                    .expect("player id");
                let bottom_seed = available_players.pop().expect("bottom seed");
                let bottom_seed_player = *players
                    .clone()
                    .get_players()
                    .get(bottom_seed - 1)
                    .expect("player id");

                this_round.push(
                    Match::new(
                        [
                            Opponent::Player(top_seed_player),
                            Opponent::Player(bottom_seed_player),
                        ],
                        [top_seed, bottom_seed],
                    )
                    .expect("match"),
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
        initial_round = false;
        i /= 2;
        available_players = (1..=i).collect();
    }

    round_matches
}

impl From<rand::Error> for Error {
    fn from(_: rand::Error) -> Self {
        Self::Rng
    }
}

impl From<PlayerError> for Error {
    fn from(e: PlayerError) -> Self {
        match e {
            PlayerError::AlreadyPresent => Self::Shuffle(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matches::Id as MatchId;
    use crate::player::Id as PlayerId;
    use rand::Rng;

    fn assert_seeding_returns_error(players: Players) {
        let e = seed(&Method::Random, players);
        assert!(e.is_err());
        if let Error::NotEnoughPlayers = e.expect_err("error") {
        } else {
            panic!("should return NotEnoughPlayers");
        }
    }

    #[test]
    fn need_at_least_three_persons_to_run_a_bracket() {
        let mut players = Players::default();
        assert_seeding_returns_error(players.clone());

        players.add(PlayerId::new_v4()).expect("player added");
        assert_seeding_returns_error(players.clone());

        players.add(PlayerId::new_v4()).expect("player added");
        assert_seeding_returns_error(players.clone());

        assert_eq!(
            players.len(),
            2,
            "there should be two players, found: {}",
            players.len()
        );
    }

    #[test]
    fn single_elimination_favors_top_seed_3_man() {
        let n = 3;
        let mut players: Vec<PlayerId> = vec![];
        (0..n).for_each(|_| {
            players.push(PlayerId::new_v4());
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego");
        let pink = players.get(1).expect("pink");
        let cute_cat = players.get(2).expect("cute_cat");

        let players = Players::from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players);
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .flatten()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*diego), Opponent::Bye],
                    [1, 4],
                    Opponent::Player(*diego),
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink), Opponent::Player(*cute_cat)],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![Match::from(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
            )
            .expect("match")],
        ];

        assert_eq!(
            matches,
            expected_matches,
            "\nexpected:\n{}\ngot:\n{}",
            RoundMatches(expected_matches.clone()),
            RoundMatches(matches.clone()),
        );
    }

    #[test]
    fn single_elimination_favors_top_seed_4_man() {
        let n = 4;
        let mut players: Vec<PlayerId> = vec![];
        (0..n).for_each(|_| {
            players.push(PlayerId::new_v4());
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego");
        let pink = players.get(1).expect("pink");
        let guy = players.get(2).expect("guy");
        let cute_cat = players.get(3).expect("cute_cat");

        let players = Players::from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players);
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .flatten()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*diego), Opponent::Player(*cute_cat)],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink), Opponent::Player(*guy)],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![Match::from(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
            )
            .expect("match")],
        ];

        assert_eq!(
            matches,
            expected_matches,
            "\nexpected:\n{}\ngot:\n{}",
            RoundMatches(expected_matches.clone()),
            RoundMatches(matches.clone()),
        );
    }

    #[test]
    fn single_elimination_favors_top_seed_5_man() {
        let n = 5;
        let mut players: Vec<PlayerId> = vec![];
        (0..n).for_each(|_| {
            players.push(PlayerId::new_v4());
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego");
        let pink = players.get(1).expect("pink");
        let average_player = players.get(2).expect("pink");
        let guy = players.get(3).expect("guy");
        let cute_cat = players.get(4).expect("cute_cat");

        let players = Players::from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players);
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .flatten()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*diego), Opponent::Bye],
                    [1, 8],
                    Opponent::Player(*diego),
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink), Opponent::Bye],
                    [2, 7],
                    Opponent::Player(*pink),
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*average_player), Opponent::Bye],
                    [3, 6],
                    Opponent::Player(*average_player),
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*guy), Opponent::Player(*cute_cat)],
                    [4, 5],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![Match::from(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
            )
            .expect("match")],
        ];

        assert_eq!(
            matches,
            expected_matches,
            "\nexpected:\n{}\ngot:\n{}",
            RoundMatches(expected_matches.clone()),
            RoundMatches(matches.clone()),
        );
    }

    #[test]
    fn single_elimination_favors_top_seed_6_man() {
        let n = 6;
        let mut players: Vec<PlayerId> = vec![];
        (0..n).for_each(|_| {
            players.push(PlayerId::new_v4());
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego");
        let pink = players.get(1).expect("pink");
        let pink_nemesis = players.get(2).expect("pink_nemesis");
        let average_player = players.get(3).expect("pink");
        let guy = players.get(4).expect("guy");
        let cute_cat = players.get(5).expect("cute_cat");

        let players = Players::from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players);
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .flatten()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*diego), Opponent::Bye],
                    [1, 8],
                    Opponent::Player(*diego),
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink), Opponent::Bye],
                    [2, 7],
                    Opponent::Player(*pink),
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink_nemesis), Opponent::Player(*cute_cat)],
                    [3, 6],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*average_player), Opponent::Player(*guy)],
                    [4, 5],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![Match::from(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
            )
            .expect("match")],
        ];

        assert_eq!(
            matches,
            expected_matches,
            "\nexpected:\n{}\ngot:\n{}",
            RoundMatches(expected_matches.clone()),
            RoundMatches(matches.clone()),
        );
    }

    #[test]
    fn single_elimination_favors_top_seed_7_man() {
        let n = 7;
        let mut players: Vec<PlayerId> = vec![];
        (0..n).for_each(|_| {
            players.push(PlayerId::new_v4());
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego");
        let pink = players.get(1).expect("pink");
        let pink_nemesis = players.get(2).expect("pink_nemesis");
        let average_player = players.get(3).expect("pink");
        let guy = players.get(4).expect("guy");
        let fg_enjoyer = players.get(5).expect("fg_enjoyer");
        let cute_cat = players.get(6).expect("cute_cat");

        let players = Players::from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players);
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .flatten()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*diego), Opponent::Bye],
                    [1, 8],
                    Opponent::Player(*diego),
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink), Opponent::Player(*cute_cat)],
                    [2, 7],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [
                        Opponent::Player(*pink_nemesis),
                        Opponent::Player(*fg_enjoyer),
                    ],
                    [3, 6],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*average_player), Opponent::Player(*guy)],
                    [4, 5],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![Match::from(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
            )
            .expect("match")],
        ];

        assert_eq!(
            matches,
            expected_matches,
            "\nexpected:\n{}\ngot:\n{}",
            RoundMatches(expected_matches.clone()),
            RoundMatches(matches.clone()),
        );
    }

    #[test]
    fn single_elimination_favors_top_seed_8_man() {
        let n = 8;
        let mut players: Vec<PlayerId> = vec![];
        (0..n).for_each(|_| {
            players.push(PlayerId::new_v4());
        });
        let players_copy = players.clone();
        let diego = players.get(0).expect("diego");
        let pink = players.get(1).expect("pink");
        let pink_nemesis = players.get(2).expect("pink_nemesis");
        let big_body_enjoyer = players.get(3).expect("big_body_enjoyer");
        let average_player = players.get(4).expect("pink");
        let guy = players.get(5).expect("guy");
        let fg_enjoyer = players.get(6).expect("fg_enjoyer");
        let cute_cat = players.get(7).expect("cute_cat");

        let players = Players::from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players);
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .flatten()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*diego), Opponent::Player(*cute_cat)],
                    [1, 8],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink), Opponent::Player(*fg_enjoyer)],
                    [2, 7],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Player(*pink_nemesis), Opponent::Player(*guy)],
                    [3, 6],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [
                        Opponent::Player(*big_body_enjoyer),
                        Opponent::Player(*average_player),
                    ],
                    [4, 5],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                )
                .expect("match"),
            ],
            vec![Match::from(
                match_ids.pop().expect("match id"),
                [Opponent::Unknown, Opponent::Unknown],
                [1, 2],
                Opponent::Unknown,
                Opponent::Unknown,
            )
            .expect("match")],
        ];

        assert_eq!(
            matches,
            expected_matches,
            "\nexpected:\n{}\ngot:\n{}",
            RoundMatches(expected_matches.clone()),
            RoundMatches(matches.clone()),
        );
    }

    #[test]
    fn single_elimination_matches_generation_does_not_break_for_with_high_entrance_numbers() {
        (0..10).for_each(|_| {
            let mut rng = rand::thread_rng();
            let n = rng.gen_range(3..3000);
            let mut players: Vec<PlayerId> = vec![];
            (0..n).for_each(|_| {
                players.push(PlayerId::new_v4());
            });
            let players = Players::from(players).expect("players");
            let players = seed(&Method::Strict, players).expect("seeded players");
            let _matches = get_balanced_round_matches_top_seed_favored(&players);
        });
    }
}
