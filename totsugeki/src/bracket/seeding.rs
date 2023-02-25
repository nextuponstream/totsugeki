//! Update seeding of bracket

use crate::{
    bracket::{Bracket, Error},
    player::{Id as PlayerId, Participants, Player},
    seeding::seed,
};

impl Bracket {
    /// Update seeding with players ordered by seeding position and generate
    /// matches
    ///
    /// # Errors
    /// thrown when provided players do not match current players in bracket
    pub fn update_seeding(self, players: &[PlayerId]) -> Result<Self, Error> {
        if self.accept_match_results {
            return Err(Error::Started(self.bracket_id, String::new()));
        }

        let mut player_group = Participants::default();
        for sorted_player in players {
            let players = self.get_participants().get_players_list();
            let Some(player) = players.iter().find(|p| p.get_id() == *sorted_player) else {
                return Err(Error::UnknownPlayer(
                    *sorted_player,
                    self.participants.clone(),
                    self.bracket_id,
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
        bracket::{raw::Raw, Id as BracketId},
        format::Format,
        matches::{Id as MatchId, Match, MatchGET},
        opponent::Opponent,
        player::{Error as PlayerError, Player},
        seeding::{
            single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
            Error as SeedingError, Method as SeedingMethod,
        },
    };
    use chrono::prelude::*;

    #[test]
    fn cannot_seed_bracket_after_it_started() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let matches = get_balanced_round_matches_top_seed_favored(&player_ids).expect("matches");
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
        let (updated_bracket, _) = bracket.start().expect("start");
        let seeding = vec![p3_id, p2_id, p1_id];
        match updated_bracket.update_seeding(&seeding) {
            Err(Error::Started(id, _)) => assert_eq!(id, bracket_id),
            Err(e) => panic!("Expected Started error, got {e}"),
            Ok(b) => panic!("Expected error, bracket: {b}"),
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
        let matches = get_balanced_round_matches_top_seed_favored(&player_ids).expect("matches");
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
        let (id, p, bracket_id) = match bracket.clone().update_seeding(&seeding) {
            Err(Error::UnknownPlayer(id, p, bracket_id)) => (id, p, bracket_id),
            Err(e) => panic!("Expected Players error, got {e}"),
            Ok(b) => panic!("Expected error, bracket: {b}"),
        };
        assert_eq!(id, unknown_player);
        assert!(p.have_same_participants(&expected_participants));
        assert_eq!(bracket_id, expected_bracket_id);

        // no players
        let seeding = vec![];
        let wrong_p = match bracket.clone().update_seeding(&seeding) {
            Err(Error::Seeding(SeedingError::DifferentParticipants(wrong_p, _actual_p))) => wrong_p,
            Err(e) => panic!(
                "Expected Error::Seeding(SeedingError::DifferentParticipants) error but got {e}"
            ),
            _ => panic!("Expected error but got none, bracket: {bracket}"),
        };
        assert!(wrong_p.is_empty());

        // duplicate player
        let seeding = vec![p1_id, p1_id, p1_id];
        match bracket.clone().update_seeding(&seeding) {
            Err(Error::PlayerUpdate(PlayerError::AlreadyPresent)) => {}
            Err(e) => panic!(
                "Expected Error::PlayerUpdate(PlayerError::AlreadyPresent) error but got {e}"
            ),
            _ => panic!("Expected error but got none, bracket: {bracket}"),
        };
    }

    #[test]
    fn updating_seeding_changes_matches_of_3_man_bracket() {
        let mut players = vec![];
        for i in 1..=3 {
            let p = Player::new(format!("p{i}"));
            players.push(p);
        }
        let matches = get_balanced_round_matches_top_seed_favored(
            &players.iter().map(Player::get_id).collect::<Vec<_>>(),
        )
        .expect("matches");
        let bracket: Bracket = Raw {
            bracket_id: BracketId::new_v4(),
            bracket_name: "bracket".to_string(),
            players: players.clone().iter().map(Player::get_id).collect(),
            player_names: players.iter().map(Player::get_name).collect(),
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
        players.reverse();
        players.push(Player::new("don't use".into())); // for readability
        players.reverse();
        let updated_bracket = bracket
            .update_seeding(&[
                players[3].get_id(),
                players[2].get_id(),
                players[1].get_id(),
            ])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(Match::get_id)
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(players[1].get_id());
        let p2 = Opponent::Player(players[2].get_id());
        let p3 = Opponent::Player(players[3].get_id());
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    &[p2, p1],
                    [2, 3],
                    &Opponent::Unknown,
                    &Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    &[p3, Opponent::Unknown],
                    [1, 2],
                    &Opponent::Unknown,
                    &Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match")
            ]
        );
    }

    #[test]
    fn updating_seeding_changes_matches_of_5_man_bracket() {
        let mut players = vec![];
        for i in 1..=5 {
            let p = Player::new(format!("p{i}"));
            players.push(p);
        }
        let matches = get_balanced_round_matches_top_seed_favored(
            &players.iter().map(Player::get_id).collect::<Vec<_>>(),
        )
        .expect("matches");
        let bracket: Bracket = Raw {
            bracket_id: BracketId::new_v4(),
            bracket_name: "bracket".to_string(),
            players: players.iter().map(Player::get_id).collect(),
            player_names: players.iter().map(Player::get_name).collect(),
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
        players.reverse();
        players.push(Player::new("don't use".into())); // for readability
        players.reverse();
        let updated_bracket = bracket
            .update_seeding(&[
                players[4].get_id(),
                players[5].get_id(),
                players[3].get_id(),
                players[2].get_id(),
                players[1].get_id(),
            ])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(Match::get_id)
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(players[1].get_id());
        let p2 = Opponent::Player(players[2].get_id());
        let p3 = Opponent::Player(players[3].get_id());
        let p4 = Opponent::Player(players[4].get_id());
        let p5 = Opponent::Player(players[5].get_id());
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    &[p2, p1],
                    [4, 5],
                    &Opponent::Unknown,
                    &Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    &[p4, Opponent::Unknown],
                    [1, 4],
                    &Opponent::Unknown,
                    &Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    &[p5, p3],
                    [2, 3],
                    &Opponent::Unknown,
                    &Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    &[Opponent::Unknown, Opponent::Unknown],
                    [1, 2],
                    &Opponent::Unknown,
                    &Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
            ]
        );
    }
}
