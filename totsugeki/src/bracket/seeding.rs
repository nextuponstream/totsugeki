//! Update seeding of bracket

use crate::{
    bracket::{Bracket, Error},
    player::{Id as PlayerId, Participants},
    seeding::{get_balanced_round_matches_top_seed_favored, seed},
};

impl Bracket {
    /// Update seeding with players ordered by seeding position and generate
    /// matches
    ///
    /// # Errors
    /// thrown when provided players do not match current players in bracket
    pub fn update_seeding(self, players: &[PlayerId]) -> Result<Self, Error> {
        if self.accept_match_results {
            return Err(Error::Started(self.bracket_id, "".into()));
        }

        let mut player_group = Participants::default();
        for sorted_player in players {
            let players = self.get_participants().get_players_list();
            let player = match players.iter().find(|p| p.get_id() == *sorted_player) {
                Some(p) => p,
                None => {
                    return Err(Error::UnknownPlayer(
                        *sorted_player,
                        self.participants.clone(),
                        self.bracket_id,
                    ))
                }
            };
            player_group = player_group.add_participant(player.clone())?;
        }
        let participants = seed(&self.seeding_method, player_group, self.participants)?;
        let matches = get_balanced_round_matches_top_seed_favored(&participants)?;
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
        bracket::{raw::Raw, Id as BracketId},
        format::Format,
        matches::{Id as MatchId, Match, MatchGET},
        opponent::Opponent,
        player::Error as PlayerError,
        seeding::{Error as SeedingError, Method as SeedingMethod},
    };
    use chrono::prelude::*;

    #[test]
    fn cannot_seed_bracket_after_it_started() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let updated_bracket = bracket.start();
        let seeding = vec![p3_id, p2_id, p1_id];
        match updated_bracket.update_seeding(&seeding) {
            Ok(b) => panic!("Expected error, bracket: {b}"),
            Err(e) => match e {
                Error::Started(id, _) => assert_eq!(id, bracket_id),
                _ => panic!("Expected Started error, got {e}"),
            },
        }
    }

    #[test]
    fn seeding_single_elimination_bracket_with_wrong_players_fails() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let unknown_player = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");

        // Unknown player
        let seeding = vec![p3_id, p2_id, unknown_player];
        let expected_participants = bracket.get_participants();
        let expected_bracket_id = bracket_id;
        match bracket.clone().update_seeding(&seeding) {
            Ok(b) => panic!("Expected error, bracket: {b}"),
            Err(e) => match e {
                Error::UnknownPlayer(id, p, bracket_id) => {
                    assert_eq!(id, unknown_player);
                    assert!(p.have_same_participants(&expected_participants));
                    assert_eq!(bracket_id, expected_bracket_id);
                }
                _ => panic!("Expected Players error, got {e}"),
            },
        };

        // no players
        let seeding = vec![];
        match bracket.clone().update_seeding(&seeding) {
            Ok(b) => panic!("Expected error, bracket: {b}"),
            Err(e) => match e {
                Error::Seeding(e) => match e {
                    SeedingError::DifferentParticipants(wrong_p, _actual_p) => {
                        assert!(wrong_p.is_empty());
                    }
                    _ => panic!("Expected DifferentParticipants error, got {e}"),
                },
                _ => panic!("Expected Seeding error, got {e}"),
            },
        };

        // duplicate player
        let seeding = vec![p1_id, p1_id, p1_id];
        match bracket.update_seeding(&seeding) {
            Ok(b) => panic!("Expected error, bracket: {b}"),
            Err(e) => match e {
                Error::PlayerUpdate(e) => match e {
                    PlayerError::AlreadyPresent => {}
                    PlayerError::PlayerId(_) => panic!("Expected AlreadyPresent error, got {e}"),
                },
                _ => panic!("Expected Seeding error, got {e}"),
            },
        };
    }
    #[test]
    fn updating_seeding_changes_matches_of_3_man_bracket() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket: Bracket = Raw {
            bracket_id: BracketId::new_v4(),
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let updated_bracket = bracket
            .update_seeding(&[p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(Match::get_id)
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(p1_id);
        let p2 = Opponent::Player(p2_id);
        let p3 = Opponent::Player(p3_id);
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p2, p1],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p3, Opponent::Unknown],
                    [1, 2],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match")
            ]
        );
    }

    #[test]
    fn updating_seeding_changes_matches_of_5_man_bracket() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let p4_id = PlayerId::new_v4();
        let p5_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id, p4_id, p5_id];
        let player_names = vec![
            "p1".to_string(),
            "p2".to_string(),
            "p3".to_string(),
            "p4".to_string(),
            "p5".to_string(),
        ];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket: Bracket = Raw {
            bracket_id: BracketId::new_v4(),
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let updated_bracket = bracket
            .update_seeding(&[p4_id, p5_id, p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(Match::get_id)
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(p1_id);
        let p2 = Opponent::Player(p2_id);
        let p3 = Opponent::Player(p3_id);
        let p4 = Opponent::Player(p4_id);
        let p5 = Opponent::Player(p5_id);
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p2, p1],
                    [4, 5],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p4, Opponent::Unknown],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p5, p3],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [1, 2],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
            ]
        );
    }
}
