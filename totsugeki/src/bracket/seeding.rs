//! Update seeding of bracket

use crate::{
    bracket::{Bracket, Error as BracketError},
    player::{Id as PlayerId, Participants, Player},
    seeding::seed,
    ID,
};
use std::collections::HashSet;
use thiserror::Error;

/// Seeding is an ordered list of player. All players IDs are guaranteed unique
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Seeding(Vec<PlayerId>);

/// Error while creating seeding
#[derive(Error, Debug, PartialEq)]
pub enum SeedingError {
    /// Duplicate player
    #[error("Duplicate player {0}")]
    DuplicatePlayer(PlayerId),
}

impl Seeding {
    /// Creates a unique player list, ordered for seeding
    pub fn new(player_ids: Vec<ID>) -> Result<Self, SeedingError> {
        let mut set = HashSet::new();
        for player_id in &player_ids {
            if !set.insert(player_id) {
                return Err(SeedingError::DuplicatePlayer(*player_id));
            }
        }
        Ok(Self(player_ids))
    }

    /// Get seeding
    pub fn get(&self) -> Vec<ID> {
        self.0.clone()
    }

    /// Contains player
    pub fn contains(&self, player_id: PlayerId) -> bool {
        self.0.contains(&player_id)
    }

    /// Number of players
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl Bracket {
    /// Update seeding with players ordered by seeding position and generate
    /// matches
    ///
    /// # Errors
    /// thrown when provided players do not match current players in bracket
    pub fn update_seeding(self, players: &[PlayerId]) -> Result<Self, BracketError> {
        if self.accept_match_results {
            return Err(BracketError::Started(self.id, String::new()));
        }

        let mut player_group = Participants::default();
        for sorted_player in players {
            let players = self.get_participants().get_players_list();
            let Some(player) = players.iter().find(|p| p.get_id() == *sorted_player) else {
                return Err(BracketError::UnknownPlayer(
                    *sorted_player,
                    self.participants.clone(),
                    self.id,
                ));
            };
            player_group = player_group.add_participant(player.clone())?;
        }
        let participants = seed(&self.seeding_method, player_group, self.participants)?;
        let matches = self.format.generate_matches(
            &participants
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<_>>(),
        )?;
        Ok(Self {
            participants,
            matches,
            ..self
        })
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bracket::builder::Builder,
        format::Format,
        matches::{Id as MatchId, Match},
        opponent::Opponent,
        player::Error as PlayerError,
        seeding::Error as OldSeedingError,
    };

    #[test]
    fn seed_many_players() {
        let players = vec![ID::new_v4(), ID::new_v4()];
        assert!(Seeding::new(players).is_ok())
    }
    #[test]
    fn seeding_throws_error_for_duplicate_id() {
        let duplicate_id = ID::new_v4();
        let players = vec![ID::new_v4(), ID::new_v4(), duplicate_id, duplicate_id];
        assert_eq!(
            Seeding::new(players),
            Err(SeedingError::DuplicatePlayer(duplicate_id))
        )
    }

    #[test]
    fn cannot_seed_bracket_after_it_started() {
        let bracket = Builder::default()
            .set_format(Format::SingleElimination)
            .set_new_players(3)
            .build()
            .expect("bracket");
        let bracket_id = bracket.id;
        let players = bracket.get_participants().get_players_list();
        let p1_id = players[0].get_id();
        let p2_id = players[1].get_id();
        let p3_id = players[2].get_id();
        let (updated_bracket, _) = bracket.start().expect("start");
        let seeding = vec![p3_id, p2_id, p1_id];
        match updated_bracket.update_seeding(&seeding) {
            Err(BracketError::Started(id, _)) => assert_eq!(id, bracket_id),
            Err(e) => panic!("Expected Started error, got {e}"),
            Ok(b) => panic!("Expected error, bracket: {b}"),
        }
    }

    #[test]
    fn seeding_single_elimination_bracket_with_wrong_players_fails() {
        let unknown_player = PlayerId::new_v4();
        let bracket = Builder::default()
            .set_format(Format::SingleElimination)
            .set_new_players(3)
            .build()
            .expect("bracket");
        let players = bracket.get_participants().get_players_list();
        let p1_id = players[0].get_id();
        let p2_id = players[1].get_id();
        let p3_id = players[2].get_id();

        // Unknown player
        let seeding = vec![p3_id, p2_id, unknown_player];
        let expected_participants = bracket.get_participants();
        let expected_bracket_id = bracket.id;
        let (id, p, bracket_id) = match bracket.clone().update_seeding(&seeding) {
            Err(BracketError::UnknownPlayer(id, p, bracket_id)) => (id, p, bracket_id),
            Err(e) => panic!("Expected Players error, got {e}"),
            Ok(b) => panic!("Expected error, bracket: {b}"),
        };
        assert_eq!(id, unknown_player);
        assert!(p.have_same_participants(&expected_participants));
        assert_eq!(bracket_id, expected_bracket_id);

        // no players
        let seeding = vec![];
        let wrong_p = match bracket.clone().update_seeding(&seeding) {
            Err(BracketError::Seeding(OldSeedingError::DifferentParticipants(
                wrong_p,
                _actual_p,
            ))) => wrong_p,
            Err(e) => panic!(
                "Expected Error::Seeding(SeedingError::DifferentParticipants) error but got {e}"
            ),
            _ => panic!("Expected error but got none, bracket: {bracket}"),
        };
        assert!(wrong_p.is_empty());

        // duplicate player
        let seeding = vec![p1_id, p1_id, p1_id];
        match bracket.clone().update_seeding(&seeding) {
            Err(BracketError::PlayerUpdate(PlayerError::AlreadyPresent)) => {}
            Err(e) => panic!(
                "Expected Error::PlayerUpdate(PlayerError::AlreadyPresent) error but got {e}"
            ),
            _ => panic!("Expected error but got none, bracket: {bracket}"),
        };
    }

    #[test]
    fn updating_seeding_changes_matches_of_3_man_bracket() {
        let bracket = Builder::default()
            .set_format(Format::SingleElimination)
            .set_new_players(3)
            .build()
            .expect("bracket");
        let players = bracket.get_participants().get_players_list();
        let p1_id = players[0].get_id();
        let p2_id = players[1].get_id();
        let p3_id = players[2].get_id();

        let updated_bracket = bracket
            .update_seeding(&[p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(Match::get_id)
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(players[0].get_id());
        let p2 = Opponent::Player(players[1].get_id());
        let p3 = Opponent::Player(players[2].get_id());
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match {
                    id: match_ids.pop().expect("match id"),
                    players: [p2, p1],
                    seeds: [2, 3],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)]
                },
                Match {
                    id: match_ids.pop().expect("match id"),
                    players: [p3, Opponent::Unknown],
                    seeds: [1, 2],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)]
                }
            ]
        );
    }

    #[test]
    fn updating_seeding_changes_matches_of_5_man_bracket() {
        let bracket = Builder::default()
            .set_format(Format::SingleElimination)
            .set_new_players(5)
            .build()
            .expect("bracket");
        let players = bracket.get_participants().get_players_list();
        let p1_id = players[0].get_id();
        let p2_id = players[1].get_id();
        let p3_id = players[2].get_id();
        let p4_id = players[3].get_id();
        let p5_id = players[4].get_id();

        let updated_bracket = bracket
            .update_seeding(&[p4_id, p5_id, p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(Match::get_id)
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(players[0].get_id());
        let p2 = Opponent::Player(players[1].get_id());
        let p3 = Opponent::Player(players[2].get_id());
        let p4 = Opponent::Player(players[3].get_id());
        let p5 = Opponent::Player(players[4].get_id());
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match {
                    id: match_ids.pop().expect("match id"),
                    players: [p2, p1],
                    seeds: [4, 5],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)]
                },
                Match {
                    id: match_ids.pop().expect("match id"),
                    players: [p4, Opponent::Unknown],
                    seeds: [1, 4],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)]
                },
                Match {
                    id: match_ids.pop().expect("match id"),
                    players: [p5, p3],
                    seeds: [2, 3],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)]
                },
                Match {
                    id: match_ids.pop().expect("match id"),
                    players: [Opponent::Unknown, Opponent::Unknown],
                    seeds: [1, 2],
                    winner: Opponent::Unknown,
                    automatic_loser: Opponent::Unknown,
                    reported_results: [(0, 0), (0, 0)]
                },
            ]
        );
    }
}
