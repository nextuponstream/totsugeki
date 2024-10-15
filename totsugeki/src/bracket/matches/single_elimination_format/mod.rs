//! Manage matches of single elimination bracket

use super::{
    assert_disqualified_at_most_once, assert_match_is_well_formed, update_bracket_with, Error,
    Progression,
};
use crate::bracket::seeding::Seeding;
use crate::bracket::Id;
use crate::{
    bracket::{disqualification::get_new_matches, progression::new_matches_to_play_for_bracket},
    matches::{Error as MatchError, Id as MatchId, Match, ReportedResult},
    opponent::Opponent,
    player::Id as PlayerId,
    seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
    ID,
};

// FIXME remove struct entirely once refactored into single_elimination_bracket
//  module
/// Computes the next step of a single-elimination tournament
#[derive(Clone, Debug)]
pub(crate) struct Step {
    /// Seeding for this bracket
    seeding: Vec<PlayerId>,
    /// All matches of single-elimination bracket
    pub(crate) matches: Vec<Match>,
    /// True when matches do not need to be validated by the tournament
    /// organiser
    automatic_progression: bool,
}

impl Step {
    /// Create new matches for single elimination bracket. If no matches are
    /// provided, generates matches with `seeding`
    ///
    /// # Errors
    /// thrown when initial matches cannot be generated
    pub fn create(seeding: &[PlayerId], automatic_progression: bool) -> Result<Self, Error> {
        let seeding = Seeding::new(seeding.to_owned()).unwrap();
        let matches = get_balanced_round_matches_top_seed_favored(seeding.clone())?;

        Ok(Self {
            matches,
            seeding: seeding.get(),
            automatic_progression,
        })
    }

    /// Create new matches for single elimination bracket.
    ///
    /// # Errors
    /// thrown when initial matches cannot be generated
    pub(crate) fn new(
        matches: Vec<Match>,
        seeding: &[PlayerId],
        automatic_progression: bool,
    ) -> Self {
        Self {
            matches,
            seeding: seeding.to_vec(),
            automatic_progression,
        }
    }

    /// Clear previous reported result for `player_id`
    fn clear_reported_result(self, player_id: PlayerId) -> Result<Self, Error> {
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
                Ok(Self { matches, ..self })
            }
            None => Err(Error::NoMatchToPlay(player_id)),
        }
    }
}

impl Step {
    fn disqualify_participant(
        &self,
        player_id: PlayerId,
    ) -> Result<(Vec<Match>, Vec<Match>), Error> {
        if self.is_over() {
            return Err(Error::TournamentIsOver);
        }

        let old_matches_to_play = self.matches_to_play();

        let Some(match_of_player_to_disqualify) = self.matches.iter().rev().find(|m| {
            m.contains(player_id)
                && m.get_winner() == Opponent::Unknown
                && m.get_automatic_loser() == Opponent::Unknown
        }) else {
            return if self.seeding.contains(&player_id) {
                Err(Error::ForbiddenDisqualified(player_id))
            } else {
                Err(Error::UnknownPlayer(player_id, self.seeding.clone()))
            };
        };
        let current_match_to_play = (*match_of_player_to_disqualify).set_automatic_loser(player_id);
        let bracket = update_bracket_with(&self.matches, &current_match_to_play);
        let bracket = Step::new(bracket, &self.seeding, self.automatic_progression);
        let new_matches = get_new_matches(&old_matches_to_play, &bracket.matches_to_play());
        match bracket
            .clone()
            .validate_match_result(current_match_to_play.get_id())
        {
            Ok((bracket, _)) => {
                let bracket = match bracket
                    .iter()
                    .copied()
                    .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
                {
                    Some(next_match_of_disqualified_player) => {
                        let match_in_losers =
                            next_match_of_disqualified_player.set_automatic_loser(player_id);
                        update_bracket_with(&bracket, &match_in_losers)
                    }
                    None => bracket,
                };
                let bracket = Step::new(bracket, &self.seeding, self.automatic_progression);

                let new_matches = get_new_matches(&old_matches_to_play, &bracket.matches_to_play());
                Ok((bracket.matches, new_matches))
            }
            // if no winner can be declared because there is a
            // missing player, then don't throw an error
            Err(Error::MatchUpdate(MatchError::MissingOpponent(_))) => {
                Ok((bracket.matches, new_matches))
            }
            Err(e) => Err(e),
        }
    }

    fn is_over(&self) -> bool {
        super::bracket_is_over(&self.matches)
    }

    fn matches_progress(&self) -> (usize, usize) {
        let right = self.matches.len();
        let left = self.matches.iter().filter(|m| m.is_over()).count();

        (left, right)
    }

    fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .iter()
            .copied()
            .filter(Match::needs_playing)
            .collect()
    }

    fn next_opponent(&self, player_id: PlayerId) -> Result<(Opponent, Id), Error> {
        if !self.seeding.contains(&player_id) {
            return Err(Error::PlayerIsNotParticipant(player_id));
        };

        if self.matches.is_empty() {
            return Err(Error::NoGeneratedMatches);
        }

        if super::is_disqualified(player_id, &self.matches) {
            return Err(Error::Disqualified(player_id));
        }

        let next_match = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let Some(relevant_match) = next_match else {
            let last_match = self.matches.iter().last().expect("last match");
            return match last_match.get_winner() {
                Opponent::Player(p) if p == player_id => Err(Error::NoNextMatch(player_id)),
                _ => Err(Error::Eliminated(player_id)),
            };
        };

        let opponent = match &relevant_match.get_players() {
            [Opponent::Player(p1), Opponent::Player(p2)] if *p1 == player_id => {
                Opponent::Player(*p2)
            }
            [Opponent::Player(p1), Opponent::Player(p2)] if *p2 == player_id => {
                Opponent::Player(*p1)
            }
            _ => Opponent::Unknown,
        };
        Ok((opponent, relevant_match.get_id()))
    }

    fn is_disqualified(&self, player_id: PlayerId) -> bool {
        super::is_disqualified(player_id, &self.matches)
    }

    fn report_result(
        &self,
        player_id: PlayerId,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, ID, Vec<Match>), Error> {
        if !self.seeding.contains(&player_id) {
            return Err(Error::UnknownPlayer(player_id, self.seeding.clone()));
        };
        if self.is_over() {
            return Err(Error::TournamentIsOver);
        }
        if self.is_disqualified(player_id) {
            return Err(Error::ForbiddenDisqualified(player_id));
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
                let p = Step::new(matches, &self.seeding, self.automatic_progression);

                let matches = if self.automatic_progression {
                    match p.clone().validate_match_result(affected_match_id) {
                        Ok((b, _)) => b,
                        Err(e) => match e {
                            Error::MatchUpdate(
                                MatchError::PlayersReportedDifferentMatchOutcome(_, _),
                            ) => p.matches,
                            Error::MatchUpdate(crate::matches::Error::MissingReport(_, _)) => {
                                p.matches
                            }
                            _ => return Err(e),
                        },
                    }
                } else {
                    p.matches
                };

                let p = Step::new(matches, &self.seeding, self.automatic_progression);

                let new_matches = p
                    .matches_to_play()
                    .iter()
                    .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
                    .map(std::clone::Clone::clone)
                    .collect();
                Ok((p.matches, affected_match_id, new_matches))
            }
            None => Err(Error::NoMatchToPlay(player_id)),
        }
    }

    fn tournament_organiser_reports_result(
        &self,
        player1: PlayerId,
        result: (i8, i8),
        player2: PlayerId,
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        let result_player_1 = ReportedResult(Some(result));
        let bracket = self.clone().clear_reported_result(player1)?;
        let bracket = bracket.clear_reported_result(player2)?;
        let (bracket, first_affected_match, _new_matches) =
            bracket.report_result(player1, result)?;
        let bracket = Step::new(bracket, &self.seeding, self.automatic_progression);
        let (bracket, second_affected_match, new_matches_2) =
            bracket.report_result(player2, result_player_1.reverse().0.expect("result"))?;
        assert_eq!(first_affected_match, second_affected_match);
        Ok((bracket, first_affected_match, new_matches_2))
    }

    fn update_player_reported_match_result(
        &self,
        match_id: MatchId,
        result: (i8, i8),
        player_id: PlayerId,
    ) -> Result<Vec<Match>, Error> {
        let Some(m) = self.matches.iter().find(|m| m.get_id() == match_id) else {
            return Err(Error::UnknownMatch(match_id));
        };

        let updated_match = (*m).update_reported_result(player_id, ReportedResult(Some(result)));
        let matches = self
            .matches
            .clone()
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match
                } else {
                    *m
                }
            })
            .collect();
        Ok(matches)
    }

    fn validate_match_result(&self, match_id: MatchId) -> Result<(Vec<Match>, Vec<Match>), Error> {
        let old_matches = self.matches.clone();
        let (matches, _) = super::update(&self.matches, match_id)?;
        let p = Step::new(matches.clone(), &self.seeding, self.automatic_progression);
        let new_matches = new_matches_to_play_for_bracket(&old_matches, &p.matches_to_play());
        Ok((matches, new_matches))
    }

    fn check_all_assertions(&self) {
        assert_disqualified_at_most_once(&self.matches, &self.seeding);
        for m in &self.matches {
            assert_match_is_well_formed(m);
        }
    }
}
