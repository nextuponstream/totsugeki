//! Generate seeded matches for single elimination

use super::{seeding_initial_round, seeding_initial_round2};
use crate::bracket::seeding::Seeding;
use crate::matches::GenerationError;
use crate::seeding::Error as SeedingError;
use crate::{matches::Match, opponent::Opponent, player::Id as PlayerId};
use thiserror::Error;

/// Single elimination bracket match generation errors
#[derive(Error, Debug)]
pub enum SingleEliminationBracketMatchGenerationError {
    /// Math overflow. If you are running a tournament with actual player that
    /// big, then congrats. If that error stops you from doing some simulation,
    /// PR are welcomed.
    #[error("Unrecoverable math overflow")]
    UnrecoverableMathOverflow,
    /// Cannot create match
    #[error("Unrecoverable match generation error {0}")]
    UnrecoverableMatchError(#[from] GenerationError),
}

/// Returns tournament matches for `n` players in a list. Used for generating
/// single elimination bracket or winner bracket in double elimination format.
///
/// Top seed plays the least matches. They will face predicted higher seeds
/// only later in the bracket. Top seed plays at most one more match than
/// anyone else.
///
/// # Errors
/// Throws error when math overflow happens
///
/// # Panics
/// We do not expect any panics here because we take the top and bottom seed to
/// form a new match or we use the players with byes and give them an "unknown"
/// Opponent.
pub(crate) fn get_balanced_round_matches_top_seed_favored2(
    seeding: &Seeding,
) -> Result<Vec<Match>, SingleEliminationBracketMatchGenerationError> {
    let player_list = seeding.get();

    // FIXME seeding should be a struct that has been well constructed
    // Matches are built bottom-up:
    // * for n
    // * compute #byes = `next_power_of_two(n)` - n
    // * for round 1, assign the #byes top seeds their bye match
    //   NOTE: you don't need to add those matches to the list of generated matches
    // * for round 1, find top+low seed, assign them a match and repeat until no players are left
    // * for round 2, select next 4 matches
    // * ...
    let n = seeding.len();
    let byes = n
        .checked_next_power_of_two()
        .ok_or(SingleEliminationBracketMatchGenerationError::UnrecoverableMathOverflow)?
        - n;
    let mut remaining_byes = byes;
    let mut this_round: Vec<Match> = vec![];
    let mut round_matches: Vec<Vec<Match>> = vec![];

    // Initialize bye matches in first round
    let mut available_players: Vec<usize> = (1..=n).collect();
    available_players.drain(0..byes); // all top seeds get a bye

    let first_round = n
        .checked_next_power_of_two()
        .ok_or(SingleEliminationBracketMatchGenerationError::UnrecoverableMathOverflow)?;
    let second_round = first_round / 2;
    let mut i = first_round;
    while i > 1 {
        while !available_players.is_empty() {
            if first_round == i {
                seeding_initial_round2(&mut available_players, seeding, &mut this_round);
            } else if second_round == i {
                let top_seed = available_players.remove(0);
                let top_seed_player = player_list[top_seed - 1];
                let bottom_seed = available_players[available_players.len() - 1];
                available_players.pop();
                let bottom_seed_player = player_list[bottom_seed - 1];
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

                this_round.push(Match::new([player_1, player_2], [top_seed, bottom_seed])?);
            } else {
                let top_seed = available_players.remove(0);
                let bottom_seed = available_players[available_players.len() - 1];
                available_players.pop();

                this_round.push(Match::new(
                    [Opponent::Unknown, Opponent::Unknown],
                    [top_seed, bottom_seed],
                )?);
            }
        }

        // empty iteration variable `this_round` into round_matches
        round_matches.push(std::mem::take(&mut this_round));
        i /= 2;
        available_players = (1..=i).collect();
    }

    Ok(round_matches.into_iter().flatten().collect())
}

/// Returns tournament matches for `n` players in a list. Used for generating
/// single elimination bracket or winner bracket in double elimination format.
///
/// Top seed plays the least matches. They will face predicted higher seeds
/// only later in the bracket. Top seed plays at most one more match than
/// anyone else.
///
/// # Errors
/// Throws error when math overflow happens
///
/// # Panics
/// We do not expect any panics here because we take the top and bottom seed to
/// form a new match, or we use the players with byes and give them an "unknown"
/// Opponent.
pub fn get_balanced_round_matches_top_seed_favored(
    seeding: &[PlayerId],
) -> Result<Vec<Match>, SeedingError> {
    // FIXME seeding should be a struct that has been well constructed
    // Matches are built bottom-up:
    // * for n
    // * compute #byes = `next_power_of_two(n)` - n
    // * for round 1, assign the #byes top seeds their bye match
    //   NOTE: you don't need to add those matches to the list of generated matches
    // * for round 1, find top+low seed, assign them a match and repeat until no players are left
    // * for round 2, select next 4 matches
    // * ...
    let n = seeding.len();
    let byes = match n.checked_next_power_of_two() {
        Some(b) => b - n,
        None => return Err(SeedingError::MathOverflow),
    };
    let mut remaining_byes = byes;
    let mut this_round: Vec<Match> = vec![];
    let mut round_matches: Vec<Vec<Match>> = vec![];

    // Initialize bye matches in first round
    let mut available_players: Vec<usize> = (1..=n).collect();
    (0..byes).for_each(|_| {
        let _top_seed = available_players.remove(0);
    });

    let Some(first_round) = n.checked_next_power_of_two() else {
        return Err(SeedingError::MathOverflow);
    };
    let second_round = first_round / 2;
    let mut i = first_round;
    while i > 1 {
        while !available_players.is_empty() {
            if first_round == i {
                seeding_initial_round(&mut available_players, seeding, &mut this_round);
            } else if second_round == i {
                let top_seed = available_players.remove(0);
                let player_list = seeding;
                let top_seed_player = player_list[top_seed - 1];
                let bottom_seed = available_players[available_players.len() - 1];
                available_players.pop();
                let bottom_seed_player = player_list[bottom_seed - 1];
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
                let bottom_seed = available_players[available_players.len() - 1];
                available_players.pop();

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
        round_matches.push(std::mem::take(&mut this_round));
        i /= 2;
        available_players = (1..=i).collect();
    }

    Ok(round_matches.into_iter().flatten().collect())
}
#[cfg(test)]
mod tests {
    use crate::matches::Id as MatchId;
    use crate::seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored;
    use crate::seeding::{seed, Method};
    use crate::{
        matches::Match,
        opponent::Opponent,
        player::{Participants, Player},
    };
    use rand::Rng;

    #[test]
    fn matches_generation_3_man_bracket() {
        let n = 3;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.remove(0);
        let pink = players.remove(0);
        let cute_cat = players.remove(0);

        let players = Participants::try_from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(
            &players
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )
        .expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches.iter().map(Match::get_id).rev().collect();
        let expected_matches = vec![
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink.get_id()),
                    Opponent::Player(cute_cat.get_id()),
                ],
                seeds: [2, 3],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Player(diego.get_id()), Opponent::Unknown],
                seeds: [1, 2],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    fn matches_generation_4_man_bracket() {
        let n = 4;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.remove(0);
        let pink = players.remove(0);
        let guy = players.remove(0);
        let cute_cat = players.remove(0);

        let players = Participants::try_from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(
            &players
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )
        .expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches
            .iter()
            .map(crate::matches::Match::get_id)
            .rev()
            .collect();
        let expected_matches = vec![
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(diego.get_id()),
                    Opponent::Player(cute_cat.get_id()),
                ],
                seeds: [1, 4],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink.get_id()),
                    Opponent::Player(guy.get_id()),
                ],
                seeds: [2, 3],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [1, 2],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    fn matches_generation_5_man_bracket() {
        let n = 5;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.remove(0);
        let pink = players.remove(0);
        let average_player = players.remove(0);
        let guy = players.remove(0);
        let cute_cat = players.remove(0);

        let players = Participants::try_from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(
            &players
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )
        .expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches.iter().map(Match::get_id).rev().collect();
        let expected_matches = vec![
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(guy.get_id()),
                    Opponent::Player(cute_cat.get_id()),
                ],
                seeds: [4, 5],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Player(diego.get_id()), Opponent::Unknown],
                seeds: [1, 4],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink.get_id()),
                    Opponent::Player(average_player.get_id()),
                ],
                seeds: [2, 3],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [1, 2],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    fn matches_generation_6_man_bracket() {
        let n = 6;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.remove(0);
        let pink = players.remove(0);
        let pink_nemesis = players.remove(0);
        let average_player = players.remove(0);
        let guy = players.remove(0);
        let cute_cat = players.remove(0);

        let players = Participants::try_from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(
            &players
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )
        .expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches.iter().map(Match::get_id).rev().collect();
        let expected_matches = vec![
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink_nemesis.get_id()),
                    Opponent::Player(cute_cat.get_id()),
                ],
                seeds: [3, 6],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(average_player.get_id()),
                    Opponent::Player(guy.get_id()),
                ],
                seeds: [4, 5],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Player(diego.get_id()), Opponent::Unknown],
                seeds: [1, 4],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Player(pink.get_id()), Opponent::Unknown],
                seeds: [2, 3],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [1, 2],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    fn matches_generation_7_man_bracket() {
        let n = 7;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.remove(0);
        let pink = players.remove(0);
        let pink_nemesis = players.remove(0);
        let average_player = players.remove(0);
        let guy = players.remove(0);
        let fg_enjoyer = players.remove(0);
        let cute_cat = players.remove(0);

        let players = Participants::try_from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(
            &players
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )
        .expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches.iter().map(Match::get_id).rev().collect();
        let expected_matches = vec![
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink.get_id()),
                    Opponent::Player(cute_cat.get_id()),
                ],
                seeds: [2, 7],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink_nemesis.get_id()),
                    Opponent::Player(fg_enjoyer.get_id()),
                ],
                seeds: [3, 6],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(average_player.get_id()),
                    Opponent::Player(guy.get_id()),
                ],
                seeds: [4, 5],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Player(diego.get_id()), Opponent::Unknown],
                seeds: [1, 4],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [2, 3],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [1, 2],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    #[allow(clippy::too_many_lines)]
    fn matches_generation_bracket_8_man() {
        let n = 8;
        let mut players: Vec<Player> = vec![];
        (0..n).for_each(|i| {
            players.push(Player::new(format!("player{i}")));
        });
        let players_copy = players.clone();
        let diego = players.remove(0);
        let pink = players.remove(0);
        let pink_nemesis = players.remove(0);
        let big_body_enjoyer = players.remove(0);
        let average_player = players.remove(0);
        let guy = players.remove(0);
        let fg_enjoyer = players.remove(0);
        let cute_cat = players.remove(0);

        let players = Participants::try_from(players_copy).expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(
            &players
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )
        .expect("balanced matches");
        let mut match_ids: Vec<MatchId> = matches.iter().map(Match::get_id).rev().collect();
        let expected_matches = vec![
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(diego.get_id()),
                    Opponent::Player(cute_cat.get_id()),
                ],
                seeds: [1, 8],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink.get_id()),
                    Opponent::Player(fg_enjoyer.get_id()),
                ],
                seeds: [2, 7],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(pink_nemesis.get_id()),
                    Opponent::Player(guy.get_id()),
                ],
                seeds: [3, 6],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [
                    Opponent::Player(big_body_enjoyer.get_id()),
                    Opponent::Player(average_player.get_id()),
                ],
                seeds: [4, 5],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [1, 4],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [2, 3],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
            Match {
                id: match_ids.pop().expect("match id"),
                players: [Opponent::Unknown, Opponent::Unknown],
                seeds: [1, 2],
                winner: Opponent::Unknown,
                automatic_loser: Opponent::Unknown,
                reported_results: [(0, 0), (0, 0)],
            },
        ];

        assert_eq!(matches, expected_matches,);
    }

    #[test]
    #[ignore]
    fn matches_generation_does_not_break_for_with_high_entrance_numbers() {
        (0..10).for_each(|_| {
            let mut rng = rand::thread_rng();
            let n = rng.gen_range(3..3000);
            let mut players: Vec<Player> = vec![];
            (0..n).for_each(|i| {
                players.push(Player::new(format!("player{i}")));
            });
            let players = Participants::try_from(players).expect("players");
            let players = seed(&Method::Strict, players.clone(), players).expect("seeded players");
            let _matches = get_balanced_round_matches_top_seed_favored(
                &players
                    .get_players_list()
                    .iter()
                    .map(Player::get_id)
                    .collect::<Vec<_>>(),
            );
        });
    }
}
