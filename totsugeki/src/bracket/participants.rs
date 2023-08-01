//! Add and remove or disqualify participants from bracket. Let them
//! join/forfeit

use super::{Bracket, Error};
use crate::player::{Id as PlayerId, Participants, Player};

impl Bracket {
    /// Regenerate matches and set participants of bracket with provided
    /// participants. Used when participants are added or removed.
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
            self.format.generate_matches(
                &updated_participants
                    .get_players_list()
                    .iter()
                    .map(Player::get_id)
                    .collect::<Vec<_>>(),
            )?
        };
        Ok(Self {
            participants: updated_participants,
            matches,
            ..self
        })
    }

    /// Let `player` join participants and returns an updated version of the bracket
    ///
    /// # Errors
    /// Thrown when bracket has already started
    pub fn join(self, player: Player) -> Result<Bracket, Error> {
        if self.is_closed {
            return Err(Error::BarredFromEntering(player.get_id(), self.get_id()));
        }
        let bracket = Self {
            participants: self.participants.clone().add_participant(player)?,
            ..self
        };
        bracket.clone().regenerate_matches(bracket.participants)
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
    use crate::{bracket::Format, seeding::Method as SeedingMethod};
    use chrono::prelude::*;

    #[test]
    fn new_participants_can_join_bracket() {
        let mut bracket = Bracket::new(
            "name",
            Format::default(),
            SeedingMethod::default(),
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
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
        let mut bracket = Bracket {
            format: Format::SingleElimination,
            ..Bracket::default()
        };
        for i in 1..=3 {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("ok");
        }

        let updated_bracket = bracket.close();
        let bracket_id = updated_bracket.get_id();

        let player = Player::new("New player".to_string());
        let player_id = player.get_id();
        let (id, b_id) = match updated_bracket.join(player) {
            Err(Error::BarredFromEntering(id, b_id)) => (id, b_id),
            Err(e) => panic!("expected BarredFromEntering error, got: {e}"),
            Ok(_) => panic!("expected error but got none"),
        };

        assert_eq!(id, player_id);
        assert_eq!(b_id, bracket_id);
    }

    #[test]
    fn starting_bracket_will_deny_new_participants_from_entering() {
        let mut bracket = Bracket {
            format: Format::SingleElimination,
            ..Bracket::default()
        };
        for i in 1..=3 {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("ok");
        }
        let (updated_bracket, _) = bracket.start().expect("start");
        let bracket_id = updated_bracket.get_id();

        let player = Player::new("New player".to_string());
        let player_id = player.get_id();
        let (id, b_id) = match updated_bracket.join(player) {
            Err(Error::BarredFromEntering(id, b_id)) => (id, b_id),
            Err(e) => panic!("expected BarredFromEntering error, got: {e}"),
            Ok(_) => panic!("expected error but got none"),
        };
        assert_eq!(id, player_id);
        assert_eq!(b_id, bracket_id);
    }
}
