//! Progression of a single elimination bracket

use crate::bracket::matches::bracket_is_over;
use crate::bracket::seeding::Seeding;
use crate::matches::{Id, Match};
use crate::opponent::Opponent;
use crate::seeding::single_elimination_seeded_bracket::{
    get_balanced_round_matches_top_seed_favored2, SingleEliminationBracketMatchGenerationError,
};
use crate::single_elimination_bracket::{
    SingleEliminationBracket, SingleEliminationReportResultError,
};
use crate::ID;
use thiserror::Error;

/// Computes the next step of a single-elimination tournament
#[derive(Clone, Debug)]
pub(crate) struct Step {
    /// Seeding for this bracket
    seeding: Seeding,
    /// All matches of single-elimination bracket
    matches: Vec<Match>,
    /// True when matches do not need to be validated by the tournament
    /// organiser
    automatic_match_progression: bool,
}

/// All errors when progressing a single elimination bracket
#[derive(Error, Debug)]
pub enum StepError {
    ///
    #[error("Unrecoverable seeding error")]
    UnrecoverableMatchGenerationError(#[from] SingleEliminationBracketMatchGenerationError),
}

// TODO for consistency, make Progression trait common to single elim and double elim but MAKE IT
//  CLEAR that the abstraction is only for library DX and it should be taken out once both
//  implementations diverge
pub trait Progression {
    // TODO force implementation of score report where you are required to tell all players involved
    //  rather then inferring (p1, p2). This way, does additional checks are done (is p2
    //  disqualified?). Currently, it only requires p1, which is fine in itself. There might be a
    //  case to require all players involved that I don't foresee, like a performance improvement
    // /// Disqualify participant from bracket and update matches. Returns updated
    // /// matches and matches to play
    // ///
    // /// # Errors
    // /// thrown when participant does not belong in tournament
    // fn disqualify_participant(
    //     &self,
    //     player_id: crate::player::Id,
    // ) -> Result<(Vec<Match>, Vec<Match>), Error>;
    //
    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    fn is_over(&self) -> bool;

    // /// Returns true if bracket is over (all matches are played)
    // #[must_use]
    // fn matches_progress(&self) -> (usize, usize);

    /// List all matches that can be played out
    fn matches_to_play(&self) -> Vec<Match>;

    // /// Return next opponent for `player_id`, relevant match and player name
    // ///
    // /// # Errors
    // /// Thrown when matches have yet to be generated or player has won/been
    // /// eliminated
    // fn next_opponent(&self, player_id: crate::player::Id) -> Result<(Opponent, crate::matches::Id), Error>;

    /// Returns true if player is disqualified
    fn is_disqualified(&self, player_id: crate::player::Id) -> bool;

    /// Report result of match. Returns updated matches, affected match and new
    /// matches to play
    /// # Errors
    /// thrown when player does not belong in bracket
    fn report_result(
        &self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), SingleEliminationReportResultError>;

    // /// Tournament organiser reports result
    // ///
    // /// NOTE: both players are needed, so it is less ambiguous when reading code:
    // /// * p1 2-0 is more ambiguous to read than
    // /// * p1 2-0 p2
    // ///
    // /// Technically, it's unnecessary.
    // ///
    // /// # Errors
    // /// thrown when player does not belong in bracket
    // fn tournament_organiser_reports_result(
    //     &self,
    //     player1: crate::player::Id,
    //     result: (i8, i8),
    //     player2: crate::player::Id,
    // ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error>;
    //
    // /// Update `match_id` with reported `result` of `player`
    // ///
    // /// # Errors
    // /// thrown when `match_id` matches no existing match
    // fn update_player_reported_match_result(
    //     &self,
    //     match_id: crate::matches::Id,
    //     result: (i8, i8),
    //     player_id: crate::player::Id,
    // ) -> Result<Vec<Match>, Error>;
    //
    // /// Returns updated matches and matches to play. Uses `match_id` as the
    // /// first match to start updating before looking deeper into the bracket
    // ///
    // /// # Errors
    // /// thrown when `match_id` matches no existing match
    // fn validate_match_result(&self, match_id: crate::matches::Id) -> Result<(Vec<Match>, Vec<Match>), Error>;
    //
    // /// Checks all assertions after updating matches
    // fn check_all_assertions(&self);
}

impl Progression for SingleEliminationBracket {
    fn is_over(&self) -> bool {
        bracket_is_over(&self.matches)
    }

    fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .iter()
            .copied()
            .filter(Match::needs_playing)
            .collect()
    }

    fn is_disqualified(&self, player_id: crate::player::Id) -> bool {
        self.matches
            .iter()
            .any(|m| m.is_automatic_loser_by_disqualification(player_id))
    }

    fn report_result(
        &self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, Id, Vec<Match>), SingleEliminationReportResultError> {
        if !self.seeding.contains(player_id) {
            return Err(SingleEliminationReportResultError::UnknownPlayer(player_id));
        };
        if self.is_over() {
            return Err(SingleEliminationReportResultError::TournamentIsOver);
        }
        if self.is_disqualified(player_id) {
            return Err(SingleEliminationReportResultError::ForbiddenDisqualified(
                player_id,
            ));
        }
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        match match_to_update {
            Some(m) => {
                let old_matches = self.matches_to_play();
                let affected_match_id = m.get_id();
                let matches =
                    self.update_player_reported_match_result(affected_match_id, result, player_id)?;
                //         let p = crate::bracket::matches::single_elimination_format::Step::new(
                //             Some(matches),
                //             &self.seeding,
                //             self.automatic_progression,
                //         )?;
                //
                //         let matches = if self.automatic_progression {
                //             match p.clone().validate_match_result(affected_match_id) {
                //                 Ok((b, _)) => b,
                //                 Err(e) => match e {
                //                     Error::MatchUpdate(
                //                         crate::matches::Error::PlayersReportedDifferentMatchOutcome(_, _),
                //                     ) => p.matches,
                //                     _ => return Err(e),
                //                 },
                //             }
                //         } else {
                //             p.matches
                //         };
                //
                //         let p = crate::bracket::matches::single_elimination_format::Step::new(
                //             Some(matches),
                //             &self.seeding,
                //             self.automatic_progression,
                //         )?;
                //
                //         let new_matches = p
                //             .matches_to_play()
                //             .iter()
                //             .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
                //             .map(std::clone::Clone::clone)
                //             .collect();
                //         Ok((p.matches, affected_match_id, new_matches))
                todo!()
            }
            None => Err(SingleEliminationReportResultError::NoMatchToPlay(player_id)),
        }
    }
}
