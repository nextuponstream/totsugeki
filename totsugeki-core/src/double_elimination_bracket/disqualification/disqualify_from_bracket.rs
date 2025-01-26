//! Disqualify from bracket. For people that cannot physically make it

use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::Match;
use crate::opponent::Opponent;
use crate::ID;
use thiserror::Error;

/// Cannot disqualify player from bracket
#[derive(Error, Debug)]
pub enum Error {
    /// Player won tournament
    #[error("Player won tournament")]
    WonTournament,
    /// Player was eliminated from tournament
    #[error("Player was eliminated from tournament")]
    Eliminated,
}

impl DoubleEliminationBracket {
    /// Disqualify participant from bracket. Returns updated bracket and new playable matches
    ///
    /// # Errors
    /// Disqualifying player is impossible at this time
    /// # Panics
    /// * player does not belong in bracket
    /// * enough player in bracket
    pub fn disqualify_participant_from_bracket(
        &self,
        player_id: ID,
    ) -> Result<(Self, Option<Vec<Match>>), Error> {
        assert!(self.seeding.contains(player_id), "player is not in bracket");
        assert!(self.seeding.len() >= 3, "enough player in bracket");

        let old_playable_matches = self.matches_to_play();
        if self.is_eliminated(player_id) {
            return Err(Error::Eliminated);
        } else if self.is_over() {
            return Err(Error::WonTournament);
        }

        let mut matches_to_update = self.matches.clone();
        let Some(m) = matches_to_update.iter_mut().rev().find(|m| {
            m.contains(player_id)
                && m.get_winner() == Opponent(None)
                && m.get_automatic_loser() == Opponent(None)
        }) else {
            panic!("Could not find match to disqualify player")
        };

        // disqualify player then validate match result to update double elimination bracket
        m.set_automatic_loser_(player_id);
        let initial_match_for_disqualification_id = m.id;
        let bracket = DoubleEliminationBracket::new(
            matches_to_update,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        // move disqualified player as far as possible
        let (bracket, _) = bracket.validate_match_result(initial_match_for_disqualification_id);

        // FIXME use next_opponent and check if opponent is already here
        let mut matches_to_update = bracket.get_matches();
        let Some(match_in_losers) = matches_to_update
            .iter_mut()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent(None))
        else {
            return Ok((
                bracket.clone(),
                bracket.new_playable_matches(&old_playable_matches),
            ));
        };
        // DQ them in loser bracket and validate result again
        match_in_losers.set_automatic_loser_(player_id);
        let match_in_loser_id = match_in_losers.id;
        let match_in_loser_can_be_validated = match_in_losers.has_all_player_reports();
        let matches_to_update = matches_to_update.clone();
        let bracket = DoubleEliminationBracket::new(
            matches_to_update.clone(),
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        if match_in_loser_can_be_validated {
            let (bracket, _) = bracket.validate_match_result(match_in_loser_id);
            Ok((
                bracket.clone(),
                bracket.new_playable_matches(&old_playable_matches),
            ))
        } else {
            Ok((
                bracket.clone(),
                bracket.new_playable_matches(&old_playable_matches),
            ))
        }
    }
}

impl DoubleEliminationBracket {
    /// Returns playable matches from previous state
    fn new_playable_matches(&self, old_playable_matches: &[Match]) -> Option<Vec<Match>> {
        let matches_to_play = self.matches_to_play();
        let new_playable_matches = matches_to_play
            .into_iter()
            .filter(|new| old_playable_matches.iter().any(|old| old.id == new.id))
            .collect::<Vec<Match>>();
        if new_playable_matches.is_empty() {
            None
        } else {
            Some(new_playable_matches)
        }
    }
}
