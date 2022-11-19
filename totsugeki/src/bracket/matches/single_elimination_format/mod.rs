//! Manage matches of single elimination bracket

mod disqualification;
mod next_opponent;
mod query_state;

use super::{update_bracket_with, Error, Progression};
use crate::{
    bracket::{disqualification::get_new_matches, progression::new_matches},
    format::Format,
    matches::{Error as MatchError, Match, ReportedResult},
    opponent::Opponent,
    player::{Id as PlayerId, Participants},
    seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
};

/// Computes the next step of a single-elimination tournament
#[derive(Clone, Debug)]
pub(crate) struct Step {
    /// Seeding for this bracket
    seeding: Participants,
    /// All matches of single-elimination bracket
    matches: Vec<Match>,
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
    pub fn new(
        matches: Option<Vec<Match>>,
        seeding: Participants,
        automatic_progression: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            matches: if let Some(m) = matches {
                m
            } else {
                get_balanced_round_matches_top_seed_favored(&seeding)?
            },
            seeding,
            automatic_progression,
        })
    }

    /// Returns true if `player_id` is disqualified
    fn is_disqualified(&self, player_id: PlayerId) -> bool {
        self.matches
            .iter()
            .any(|m| m.is_automatic_loser_by_disqualification(player_id))
    }

    /// Clear previous reported result for `player_id`
    fn clear_reported_result(self, player_id: PlayerId) -> Result<Self, Error> {
        let match_to_update = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let participants = self.seeding.get_players_list();
        let player = participants
            .iter()
            .find(|p| p.get_id() == player_id)
            .expect("player");
        match match_to_update {
            Some(m_to_clear) => {
                let m_to_clear = m_to_clear.clone().clear_reported_result(player_id);

                let matches = self
                    .matches
                    .into_iter()
                    .map(|m| {
                        if m.get_id() == m_to_clear.get_id() {
                            m_to_clear.clone()
                        } else {
                            m
                        }
                    })
                    .collect();
                Ok(Self { matches, ..self })
            }
            None => Err(Error::NoMatchToPlay(player.clone())),
        }
    }
}

impl Progression for Step {
    fn disqualify_participant(
        &self,
        player_id: crate::player::Id,
    ) -> Result<(Vec<Match>, Vec<Match>), Error> {
        if self.is_over() {
            return Err(Error::TournamentIsOver);
        }

        let old_matches_to_play = self.matches_to_play();

        if let Some(m) = self.matches.iter().rev().find(|m| {
            m.contains(player_id)
                && m.get_winner() == Opponent::Unknown
                && m.get_automatic_loser() == Opponent::Unknown
        }) {
            // same as match in winners
            let current_match_to_play = m.clone().set_automatic_loser(player_id)?;
            let bracket = update_bracket_with(&self.matches, &current_match_to_play);
            let p = Step::new(
                Some(bracket),
                self.seeding.clone(),
                self.automatic_progression,
            )?;
            match p
                .clone()
                .validate_match_result(current_match_to_play.get_id())
            {
                Ok((bracket, _)) => {
                    let b = if let Some(m) = bracket
                        .iter()
                        .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
                    {
                        let match_in_losers = m.clone().set_automatic_loser(player_id)?;
                        update_bracket_with(&bracket, &match_in_losers)
                    } else {
                        bracket.clone()
                    };

                    let p = Step::new(
                        Some(bracket),
                        self.seeding.clone(),
                        self.automatic_progression,
                    )?;
                    let new_matches = get_new_matches(&old_matches_to_play, &p.matches_to_play());
                    Ok((b, new_matches))
                }
                Err(bracket_e) => {
                    if let Error::MatchUpdate(ref e) = bracket_e {
                        match e {
                            // if no winner can be declared because there is a
                            // missing player, then don't throw an error
                            MatchError::MissingOpponent(_) => {
                                let new_matches =
                                    get_new_matches(&old_matches_to_play, &p.matches_to_play());
                                Ok((p.matches, new_matches))
                            }
                            _ => Err(bracket_e),
                        }
                    } else {
                        Err(bracket_e)
                    }
                }
            }
        } else {
            if let Some(p) = self.seeding.get(player_id) {
                return Err(Error::PlayerDisqualified(p));
            }
            Err(Error::UnknownPlayer(player_id, self.seeding.clone()))
        }
    }

    fn get_format(&self) -> Format {
        Format::SingleElimination
    }

    fn is_over(&self) -> bool {
        super::bracket_is_over(&self.matches)
    }

    fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .iter()
            .cloned()
            .filter(Match::is_playable)
            .collect()
    }

    fn next_opponent(
        &self,
        player_id: crate::player::Id,
    ) -> Result<(crate::opponent::Opponent, crate::matches::Id, String), Error> {
        let player = if let Some(p) = self.seeding.get(player_id) {
            p
        } else {
            return Err(Error::PlayerIsNotParticipant(player_id));
        };

        if self.matches.is_empty() {
            return Err(Error::NoGeneratedMatches);
        }

        if super::is_disqualified(player_id, &self.matches) {
            return Err(Error::Disqualified(player));
        }

        let next_match = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let relevant_match = if let Some(m) = next_match {
            m
        } else {
            let last_match = self.matches.iter().last().expect("last match");
            if let Opponent::Player(p) = last_match.get_winner() {
                if p.get_id() == player_id {
                    return Err(Error::NoNextMatch(p));
                }
            }
            return Err(Error::Eliminated(player));
        };

        let mut opponent = Opponent::Unknown;
        if let Opponent::Player(p) = &relevant_match.get_players()[0] {
            if p.get_id() == player_id {
                opponent = relevant_match.get_players()[1].clone();
            }
        }
        if let Opponent::Player(p) = &relevant_match.get_players()[1] {
            if p.get_id() == player_id {
                opponent = relevant_match.get_players()[0].clone();
            }
        }
        Ok((opponent, relevant_match.get_id(), player.get_name()))
    }

    fn report_result(
        &self,
        player_id: crate::player::Id,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        let player = if let Some(p) = self.seeding.get(player_id) {
            p
        } else {
            return Err(Error::UnknownPlayer(player_id, self.seeding.clone()));
        };
        if self.is_over() {
            return Err(Error::TournamentIsOver);
        }
        if self.is_disqualified(player_id) {
            return Err(Error::PlayerDisqualified(player));
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
                    Some(matches),
                    self.seeding.clone(),
                    self.automatic_progression,
                )?;

                let matches = if self.automatic_progression {
                    match p.clone().validate_match_result(affected_match_id) {
                        Ok((b, _)) => b,
                        Err(e) => match e {
                            Error::MatchUpdate(
                                MatchError::PlayersReportedDifferentMatchOutcome(_, _),
                            ) => p.matches,
                            _ => return Err(e),
                        },
                    }
                } else {
                    p.matches
                };

                let p = Step::new(
                    Some(matches),
                    self.seeding.clone(),
                    self.automatic_progression,
                )?;

                let new_matches = p
                    .matches_to_play()
                    .iter()
                    .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
                    .map(std::clone::Clone::clone)
                    .collect();
                Ok((p.matches, affected_match_id, new_matches))
            }
            None => Err(Error::NoMatchToPlay(player)),
        }
    }

    fn tournament_organiser_reports_result(
        &self,
        player1: PlayerId,
        result: (i8, i8),
        player2: PlayerId,
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        let result_player_1 = ReportedResult(result);
        let matches = self.clone().clear_reported_result(player1)?;
        let matches = matches.clear_reported_result(player2)?;
        let (matches, first_affected_match, _new_matches) =
            matches.report_result(player1, result_player_1.0)?;
        let progression = Step::new(
            Some(matches),
            self.seeding.clone(),
            self.automatic_progression,
        )?;
        let (matches, second_affected_match, new_matches_2) =
            progression.report_result(player2, result_player_1.reverse().0)?;
        assert_eq!(first_affected_match, second_affected_match);
        Ok((matches, first_affected_match, new_matches_2))
    }

    fn update_player_reported_match_result(
        &self,
        match_id: crate::matches::Id,
        result: (i8, i8),
        player_id: crate::player::Id,
    ) -> Result<Vec<Match>, Error> {
        let m = match self.matches.iter().find(|m| m.get_id() == match_id) {
            Some(m) => m,
            None => return Err(Error::UnknownMatch(match_id)),
        };

        let updated_match = m
            .clone()
            .update_reported_result(player_id, ReportedResult(result))?;
        let matches = self
            .matches
            .clone()
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match.clone()
                } else {
                    m.clone()
                }
            })
            .collect();
        Ok(matches)
    }

    fn validate_match_result(
        &self,
        match_id: crate::matches::Id,
    ) -> Result<(Vec<Match>, Vec<Match>), Error> {
        let old_matches = self.matches_to_play();
        let (matches, _) = super::update(&self.matches, match_id)?;
        let p = Step::new(
            Some(matches.clone()),
            self.seeding.clone(),
            self.automatic_progression,
        )?;
        let new_matches = new_matches(&old_matches, &p.matches_to_play());
        Ok((matches, new_matches))
    }

    fn is_disqualified(&self, player_id: PlayerId) -> bool {
        super::is_disqualified(player_id, &self.matches)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bracket::matches::{
            single_elimination_format::{self, Step},
            Progression,
        },
        opponent::Opponent,
        player::{Id as PlayerId, Participants, Player},
    };

    fn assert_players_play_each_other(
        player_1: usize,
        player_2: usize,
        player_ids: &[PlayerId],
        matches: &Step,
    ) {
        let (next_opponent, match_id_1, _msg) = matches
            .next_opponent(player_ids[player_1])
            .expect("next opponent");
        if let Opponent::Player(next_opponent) = next_opponent {
            assert_eq!(next_opponent.get_id(), player_ids[player_2]);
        } else {
            panic!("expected player")
        }
        let (next_opponent, match_id_2, _msg) = matches
            .next_opponent(player_ids[player_2])
            .expect("next opponent");
        if let Opponent::Player(next_opponent) = next_opponent {
            assert_eq!(next_opponent.get_id(), player_ids[player_1]);
        } else {
            panic!("expected player")
        }

        assert_eq!(
            match_id_1, match_id_2,
            "expected player to be playing the same match"
        );
    }

    #[test]
    fn player_reports_before_tournament_organiser() {
        // player 2 reports before TO does
        let mut seeding = Participants::default();
        let mut player_ids = vec![PlayerId::new_v4()]; // padding
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("updated seeding");
        }
        let automatic_progression = true;
        let matches = Step::new(None, seeding.clone(), automatic_progression).expect("matches");

        assert_players_play_each_other(2, 3, &player_ids, &matches);
        let (matches, _, _) = matches
            .report_result(player_ids[2], (2, 0))
            .expect("matches");
        let p = Step::new(Some(matches), seeding, automatic_progression).expect("progression");
        let (_, _, _) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("matches");

        // player 3 reports before TO does
        let mut seeding = Participants::default();
        let mut player_ids = vec![PlayerId::new_v4()]; // padding
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("updated seeding");
        }
        let matches = Step::new(None, seeding.clone(), true).expect("matches");

        assert_players_play_each_other(2, 3, &player_ids, &matches);
        let (matches, _, _) = matches
            .report_result(player_ids[3], (0, 2))
            .expect("matches");
        let p = Step::new(Some(matches), seeding, automatic_progression).expect("progression");
        let (_, _, _) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("matches");
    }

    #[test]
    fn run_3_man_bracket() {
        let mut seeding = Participants::default();
        let mut player_ids = vec![PlayerId::new_v4()]; // padding
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("updated seeding");
        }
        let automatic_progression = true;
        let matches = Step::new(None, seeding.clone(), automatic_progression).expect("matches");

        assert_eq!(matches.matches.len(), 2);
        assert_eq!(matches.matches_to_play().len(), 1);
        assert_players_play_each_other(2, 3, &player_ids, &matches);
        let (matches, _, new_matches) = matches
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("matches");
        let p = single_elimination_format::Step::new(
            Some(matches.clone()),
            seeding.clone(),
            automatic_progression,
        )
        .expect("progress");
        assert!(matches[0].get_winner() != Opponent::Unknown);
        assert_eq!(new_matches.len(), 1, "grand finals match generated");
        assert_players_play_each_other(1, 2, &player_ids, &p);
        assert_eq!(p.matches_to_play().len(), 1);
        let (matches, _, new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
            .expect("matches");
        let p = single_elimination_format::Step::new(Some(matches), seeding, automatic_progression)
            .expect("progress");
        assert!(p.matches_to_play().is_empty());
        assert!(new_matches.is_empty());
        assert!(p.is_over());
    }

    #[test]
    fn run_5_man_bracket() {
        let mut seeding = Participants::default();
        let mut player_ids = vec![PlayerId::new_v4()]; // padding
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("updated seeding");
        }
        let automatic_progression = true;

        let p = Step::new(None, seeding.clone(), automatic_progression).expect("progress");
        assert_eq!(p.matches.len(), 4);
        assert_eq!(p.matches_to_play().len(), 2);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progress");
        assert_eq!(p.matches_to_play().len(), 2);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progress");
        assert_eq!(p.matches_to_play().len(), 1);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progress");
        assert_eq!(p.matches_to_play().len(), 1);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding, automatic_progression).expect("progress");
        if !p.is_over() {
            for m in p.matches {
                println!("{m}");
            }
            panic!("expected bracket to be over")
        }
        assert_eq!(p.matches_to_play().len(), 0);
    }
}
