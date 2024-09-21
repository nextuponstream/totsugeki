//! Single elimination bracket

mod progression;

use crate::bracket::matches::single_elimination_format::Step;
use crate::bracket::matches::{Error, Progression};
use crate::bracket::seeding::Seeding;
use crate::matches::Match;
use crate::opponent::Opponent;
use crate::opponent::Opponent::Player;
use crate::seeding::Error as SeedingError;
use crate::single_elimination_bracket::progression::ProgressionSEB;
use crate::ID;
use thiserror::Error;

/// Single elimination bracket
#[derive(Clone)]
pub struct SingleEliminationBracket {
    /// Matches
    matches: Vec<Match>,
    /// Seeding
    seeding: Seeding,
    /// True when a match should not require tournament organiser to be finalized
    automatic_match_progression: bool,
}

/// All errors you might come across when players reports match result
#[derive(Error, Debug)]
pub enum SingleEliminationReportResultError {
    #[error("Cannot join single elimination bracket because of unrecoverable seeding error {0}")]
    /// Seeding is wrong
    UnrecoverableSeedingError(#[from] SeedingError),
    /// Player is unknown, user provided a wrong player
    #[error("Player {0} is unknown")]
    UnknownPlayer(ID),
    /// Match is unknown, user provided a wrong match
    #[error("Match {0} is unknown")]
    UnknownMatch(ID),
    /// Tournament is already over
    #[error("Tournament is over")]
    TournamentIsOver,
    /// Player is disqualified
    #[error("Player {0} is disqualified")]
    ForbiddenDisqualified(ID),
    /// No match to play for player
    #[error("There is no matches for player {0}")]
    NoMatchToPlay(ID),
    /// Missing opponent
    #[error("Missing opponent")]
    MissingOpponent(),
}

/// Cannot generate single elimination bracket
#[derive(Error, Debug)]
pub enum SingleEliminationBracketGenerationError {
    /// Unknown
    #[error("Seeding does not contain player {0} present in match {1}")]
    UnknownPlayer(ID, ID),
}

impl SingleEliminationBracket {
    /// New single elimination bracket
    pub fn new(
        seeding: Seeding,
        matches: Vec<Match>,
        automatic_match_progression: bool,
    ) -> Result<Self, SingleEliminationBracketGenerationError> {
        // NOTE: I really don't like taking `matches` without verifying anything whatsoever
        // FIXME add some assertions, could save from a grave mistake, example: any players not in
        // seeding found in matches should cause an unrecoverable error
        // NOTE: could make it "fumble proof" by only recording reports, but then you have to
        // recompute the bracket at every turn. Then it's not efficient. Just use the database for
        // what it is, saving intermediate state
        for player in seeding.get() {
            // could downgrade to debug_assert but let's verify assumptions, even in release mode
            assert!(matches
                .iter()
                .find(|m| m.players.contains(&Player(player)))
                .is_some());
        }

        Ok(Self {
            seeding,
            matches,
            automatic_match_progression,
        })
    }

    /// Report result for a match in this bracket. Returns updated bracket,
    /// match id where result is reported and new generated matches if
    /// automatic match validation is on.
    ///
    /// # Errors
    /// thrown when result cannot be parsed
    pub fn report_result(
        self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(SingleEliminationBracket, ID, Vec<Match>), SingleEliminationReportResultError>
    {
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
        let old_matches = self.matches_to_play();
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        match match_to_update {
            Some(m) => {
                let affected_match_id = m.get_id();
                let matches =
                    self.update_player_reported_match_result(affected_match_id, result, player_id)?;
                let p = Step::new(
                    matches,
                    &self.seeding.get(),
                    self.automatic_match_progression,
                );

                let matches = if self.automatic_match_progression {
                    // FIXME update error type is probably too big
                    match p.clone().validate_match_result(affected_match_id) {
                        Ok((b, _)) => b,
                        Err(e) => match e {
                            Error::MatchUpdate(
                                crate::matches::Error::PlayersReportedDifferentMatchOutcome(_, _),
                            ) => p.matches,
                            Error::MatchUpdate(crate::matches::Error::MissingReport(_, _)) => {
                                p.matches
                            }
                            _ => unreachable!(),
                        },
                    }
                } else {
                    p.matches
                };

                let p = Step::new(
                    matches,
                    &self.seeding.get(),
                    self.automatic_match_progression,
                );

                let new_matches = p
                    .matches_to_play()
                    .iter()
                    .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
                    .map(std::clone::Clone::clone)
                    .collect();
                let seb = SingleEliminationBracket::new(
                    self.seeding,
                    p.matches,
                    self.automatic_match_progression,
                )
                .expect("single elimination bracket");
                Ok((seb, affected_match_id, new_matches))
            }
            None => Err(SingleEliminationReportResultError::NoMatchToPlay(player_id)),
        }
    }

    /// Clear previous reported result for `player_id`
    fn clear_reported_result(self, player_id: ID) -> Self {
        debug_assert!(
            self.matches
                .iter()
                .filter(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
                .count()
                <= 1
        );
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        match match_to_update {
            Some(m_to_clear) => {
                let m_to_clear = (*m_to_clear).clear_reported_result(player_id);

                let matches = self
                    .matches
                    .into_iter()
                    .map(|m| {
                        if m.get_id() == m_to_clear.get_id() {
                            m_to_clear
                        } else {
                            m
                        }
                    })
                    .collect();
                Self { matches, ..self }
            }
            None => self,
        }
    }
}
