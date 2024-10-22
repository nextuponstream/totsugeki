//! Disqualify from bracket. For people that cannot physically make it

use crate::bracket::matches::update_bracket_with;
use crate::double_elimination_bracket::progression::ProgressionDEB;
use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::{
    double_elimination_matches_from_partition, partition_double_elimination_matches, Match,
};
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
    /// Disqualify participant from bracket and update matches
    ///
    /// # Errors
    /// Disqualifying player is impossible at this time
    /// # Panics
    /// * player does not belong in bracket
    pub fn disqualify_participant_from_bracket(&self, player_id: ID) -> Result<Self, Error> {
        assert!(self.seeding.contains(player_id), "player is not in bracket");
        if self.is_eliminated(player_id) {
            return Err(Error::Eliminated);
        } else if self.is_over() {
            return Err(Error::WonTournament);
        }
        let disqualified = player_id;

        let mut matches_to_update = self.matches.clone();
        let Some(m) = matches_to_update.iter_mut().rev().find(|m| {
            m.contains(player_id)
                && m.get_winner() == Opponent::Unknown
                && m.get_automatic_loser() == Opponent::Unknown
        }) else {
            panic!("Could not find match to disqualify player")
        };

        // disqualify player then validate match result to update double elimination bracket
        m.set_automatic_loser_(player_id);
        let initial_match_for_disqualification_can_be_validated = m.both_opponents_are_present();
        let expected_loser_seed = m.get_seeds()[1];
        let initial_match_for_disqualification_id = m.id;
        let bracket = DoubleEliminationBracket::new(
            matches_to_update,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        let (w_bracket, l_bracket, gf, gf_reset) =
            partition_double_elimination_matches(&bracket.get_matches(), self.seeding.len());
        // don't send to loser if the disqualified player is in gf or gf_reset
        let l_bracket = if gf.contains(disqualified)
            || gf.contains(disqualified)
            || l_bracket.iter().any(|m| m.contains(disqualified))
        {
            l_bracket
        } else {
            send_to_losers(&l_bracket, disqualified, expected_loser_seed)
        };
        let matches =
            double_elimination_matches_from_partition(&w_bracket, &l_bracket, gf, gf_reset);
        if initial_match_for_disqualification_can_be_validated {
            // move disqualified player as far as possible
            let (bracket, _) = bracket.validate_match_result(initial_match_for_disqualification_id);

            // FIXME use next_opponent and check if opponent is already here
            let mut matches_to_update = bracket.get_matches();
            let Some(match_in_losers) = matches_to_update
                .iter_mut()
                .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
            else {
                return Ok(bracket);
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
                Ok(bracket)
            } else {
                Ok(bracket)
            }
        } else {
            Ok(bracket)
        }
        // Err(bracket_e) => {
        //     // if no winner can be declared because there is a
        //     // missing player, then don't throw an error
        //     let Error::MatchUpdate(ref e) = bracket_e else {
        //         return Err(bracket_e);
        //     };
        //     match e {
        //         MatchError::MissingOpponent(_) => {
        //             disqualify_player(&p, player_id, &old_matches)
        //         }
        //         MatchError::PlayersReportedDifferentMatchOutcome(_, _) => {
        //             // Can't update match in losers where disqualified player is in.
        //             // Set disqualified player as loser and update
        //             disqualify_player_and_update_bracket(
        //                 &p,
        //                 player_id,
        //                 &self.seeding,
        //                 self.auto,
        //                 &old_matches,
        //             )
        //         }
        //         _ => Err(bracket_e),
        //     }
        // }
    }
}

/// Place loser from winner's bracket into loser bracket using seed of
/// `expected_loser_seed`. Returns updated loser bracket
fn send_to_losers(
    loser_bracket: &[Match],
    loser: crate::player::Id,
    expected_loser_seed: usize,
) -> Vec<Match> {
    let loser_match = loser_bracket
        .iter()
        .find(|m| m.is_first_loser_match(expected_loser_seed))
        .expect("match");
    let is_player_1 = expected_loser_seed == loser_match.get_seeds()[0];
    let loser_match = (*loser_match).insert_player(loser, is_player_1);

    update_bracket_with(loser_bracket, &loser_match)
}
