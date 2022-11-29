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
    player::{Id as PlayerId, Participants, Player},
    seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
};

/// Computes the next step of a single-elimination tournament
#[derive(Clone, Debug)]
pub(crate) struct Step {
    // FIXME remove
    /// Seeding for this bracket but it does not need player names
    bad_seeding: Participants,
    /// Seeding for this bracket
    seeding: Vec<PlayerId>,
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
        bad_seeding: Participants,
        seeding: Vec<PlayerId>,
        automatic_progression: bool,
    ) -> Result<Self, Error> {
        Ok(Self {
            matches: if let Some(m) = matches {
                m
            } else {
                get_balanced_round_matches_top_seed_favored(&bad_seeding, &seeding)?
            },
            seeding: bad_seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect::<Vec<PlayerId>>(),

            bad_seeding,
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
            None => Err(Error::NoMatchToPlay(player_id)),
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

        let Some(match_of_player_to_disqualify) = self.matches.iter().rev().find(|m| {
            m.contains(player_id)
                && m.get_winner() == Opponent::Unknown
                && m.get_automatic_loser() == Opponent::Unknown
        }) else {
            return match self.bad_seeding.get(player_id) {
                Some(_) => Err(Error::ForbiddenDisqualified(player_id)),
                None => Err(Error::UnknownPlayer(player_id, self.bad_seeding.clone())),
            }
        };
        let current_match_to_play = match_of_player_to_disqualify
            .clone()
            .set_automatic_loser(player_id)?;
        let bracket = update_bracket_with(&self.matches, &current_match_to_play);
        let bracket = Step::new(
            Some(bracket),
            self.bad_seeding.clone(),
            self.seeding.clone(),
            self.automatic_progression,
        )?;
        let new_matches = get_new_matches(&old_matches_to_play, &bracket.matches_to_play());
        match bracket
            .clone()
            .validate_match_result(current_match_to_play.get_id())
        {
            Ok((bracket, _)) => {
                let bracket = match bracket
                    .iter()
                    .cloned()
                    .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
                {
                    Some(next_match_of_disqualified_player) => {
                        let match_in_losers =
                            next_match_of_disqualified_player.set_automatic_loser(player_id)?;
                        update_bracket_with(&bracket, &match_in_losers)
                    }
                    None => bracket,
                };
                let bracket = Step::new(
                    Some(bracket),
                    self.bad_seeding.clone(),
                    self.seeding.clone(),
                    self.automatic_progression,
                )?;

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
        let Some(player) = self.bad_seeding.get(player_id) else {
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
                Opponent::Player(p) if p.get_id() == player_id => Err(Error::NoNextMatch(player_id)),
                _ => Err(Error::Eliminated(player.get_id())),
            }
        };

        let opponent = match &relevant_match.get_players() {
            [Opponent::Player(p1), Opponent::Player(p2)] if p1.get_id() == player_id => {
                Opponent::Player(p2.clone())
            }
            [Opponent::Player(p1), Opponent::Player(p2)] if p2.get_id() == player_id => {
                Opponent::Player(p1.clone())
            }
            _ => Opponent::Unknown,
        };
        Ok((opponent, relevant_match.get_id(), player.get_name()))
    }

    fn report_result(
        &self,
        player_id: crate::player::Id,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        if self.bad_seeding.get(player_id).is_none() {
            return Err(Error::UnknownPlayer(player_id, self.bad_seeding.clone()));
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
                let p = Step::new(
                    Some(matches),
                    self.bad_seeding.clone(),
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
                    self.bad_seeding.clone(),
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
            None => Err(Error::NoMatchToPlay(player_id)),
        }
    }

    fn tournament_organiser_reports_result(
        &self,
        player1: PlayerId,
        result: (i8, i8),
        player2: PlayerId,
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        let result_player_1 = ReportedResult(result);
        let bracket = self.clone().clear_reported_result(player1)?;
        let bracket = bracket.clear_reported_result(player2)?;
        let (bracket, first_affected_match, _new_matches) =
            bracket.report_result(player1, result_player_1.0)?;
        let bracket = Step::new(
            Some(bracket),
            self.bad_seeding.clone(),
            self.seeding.clone(),
            self.automatic_progression,
        )?;
        let (bracket, second_affected_match, new_matches_2) =
            bracket.report_result(player2, result_player_1.reverse().0)?;
        assert_eq!(first_affected_match, second_affected_match);
        Ok((bracket, first_affected_match, new_matches_2))
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
            self.bad_seeding.clone(),
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
        let bracket = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            automatic_progression,
        )
        .expect("matches");

        assert_players_play_each_other(2, 3, &player_ids, &bracket);
        let (matches, _, _) = bracket
            .report_result(player_ids[2], (2, 0))
            .expect("matches");
        let p = Step::new(
            Some(matches),
            seeding,
            bracket.seeding,
            automatic_progression,
        )
        .expect("progression");
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
        let matches = Step::new(None, seeding.clone(), p.seeding.clone(), true).expect("matches");

        assert_players_play_each_other(2, 3, &player_ids, &matches);
        let (matches, _, _) = matches
            .report_result(player_ids[3], (0, 2))
            .expect("matches");
        let p = Step::new(Some(matches), seeding, p.seeding, automatic_progression)
            .expect("progression");
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
        let bracket = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            automatic_progression,
        )
        .expect("matches");

        assert_eq!(bracket.matches.len(), 2);
        assert_eq!(bracket.matches_to_play().len(), 1);
        assert_players_play_each_other(2, 3, &player_ids, &bracket);
        let (matches, _, new_matches) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("matches");
        let p = single_elimination_format::Step::new(
            Some(matches.clone()),
            seeding.clone(),
            bracket.seeding,
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
        let p = single_elimination_format::Step::new(
            Some(matches),
            seeding,
            p.seeding,
            automatic_progression,
        )
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

        let p = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            automatic_progression,
        )
        .expect("progress");
        assert_eq!(p.matches.len(), 4);
        assert_eq!(p.matches_to_play().len(), 2);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(
            Some(matches),
            seeding.clone(),
            p.seeding,
            automatic_progression,
        )
        .expect("progress");
        assert_eq!(p.matches_to_play().len(), 2);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(
            Some(matches),
            seeding.clone(),
            p.seeding,
            automatic_progression,
        )
        .expect("progress");
        assert_eq!(p.matches_to_play().len(), 1);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
            .expect("bracket");
        let p = Step::new(
            Some(matches),
            seeding.clone(),
            p.seeding,
            automatic_progression,
        )
        .expect("progress");
        assert_eq!(p.matches_to_play().len(), 1);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding, p.seeding, automatic_progression).expect("progress");
        if !p.is_over() {
            for m in p.matches {
                println!("{m}");
            }
            panic!("expected bracket to be over")
        }
        assert_eq!(p.matches_to_play().len(), 0);
    }
}
