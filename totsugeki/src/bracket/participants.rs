//! Add and remove or disqualify participants from bracket. Let them
//! join/forfeit

use super::{Bracket, Error};
use crate::{
    player::{Id as PlayerId, Participants, Player},
    seeding::get_balanced_round_matches_top_seed_favored,
};

impl Bracket {
    /// Regenerate matches. Used when participants are added or removed
    ///
    /// # Errors
    /// thrown when math overflow happens
    pub(super) fn regenerate_matches(
        self,
        updated_participants: Participants,
    ) -> Result<Self, Error> {
        let matches = if updated_participants.len() < 3 {
            vec![]
        } else {
            get_balanced_round_matches_top_seed_favored(&updated_participants)?
        };
        Ok(Self {
            participants: updated_participants,
            matches,
            ..self
        })
    }

    /// Adds new player in participants and returns updated bracket
    ///
    /// # Errors
    /// thrown when the same player is added
    pub fn add_new_player(self, player: Player) -> Result<Bracket, Error> {
        let updated_participants = self.participants.clone().add_participant(player)?;
        self.regenerate_matches(updated_participants)
    }

    /// Let `player` join participants and returns an updated version of the bracket
    ///
    /// # Errors
    /// Thrown when bracket has already started
    pub fn join(self, player: Player) -> Result<Bracket, Error> {
        if self.is_closed {
            return Err(Error::BarredFromEntering(player.get_id(), self.get_id()));
        }
        let updated_bracket = self.add_new_player(player)?;
        Ok(updated_bracket)
    }

    /// Remove participant, regenerate matches and return updated bracket
    ///
    /// # Errors
    /// thrown if referred participant does not belong in bracket
    pub fn remove_participant(self, participant_id: PlayerId) -> Result<Self, Error> {
        if self.accept_match_results {
            return Err(Error::Started(
                self.bracket_id,
                ". As a player, you can quit the bracket by forfeiting or ask an admin to disqualify you."
                    .into(),
            ));
        }
        let updated_participants = self.participants.clone().remove(participant_id);
        self.regenerate_matches(updated_participants)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bracket::{raw::Raw, Format, Id as BracketId},
        player::Participants,
        seeding::{get_balanced_round_matches_top_seed_favored, Method as SeedingMethod},
    };
    use chrono::prelude::*;

    #[test]
    fn new_participants_can_join_bracket() {
        let mut bracket = Bracket::new(
            "name",
            Format::default(),
            SeedingMethod::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        );
        for i in 0..10 {
            bracket = bracket
                .join(Player::new(format!("player{i}")))
                .expect("updated_bracket");
        }
    }

    #[test]
    fn closing_bracket_will_deny_new_participants_from_entering() {
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
        let updated_bracket = bracket.close();
        let bracket_id = updated_bracket.get_id();

        let player = Player::new("New player".to_string());
        let player_id = player.get_id();
        let err = updated_bracket
            .join(player)
            .expect_err("Joining a bracket after closing it did not return an error");
        match err {
            Error::BarredFromEntering(id, b_id) => {
                assert_eq!(id, player_id);
                assert_eq!(b_id, bracket_id);
            }
            _ => panic!("expected BarredFromEntering error, got: {}", err),
        };
    }

    #[test]
    fn starting_bracket_will_deny_new_participants_from_entering() {
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
        let updated_bracket = bracket.start();
        let bracket_id = updated_bracket.get_id();

        let player = Player::new("New player".to_string());
        let player_id = player.get_id();
        let err = updated_bracket
            .join(player)
            .expect_err("Joining a bracket after closing it did not return an error");
        match err {
            Error::BarredFromEntering(id, b_id) => {
                assert_eq!(id, player_id);
                assert_eq!(b_id, bracket_id);
            }
            _ => panic!("expected BarredFromEntering error, got: {}", err),
        };
    }
}
