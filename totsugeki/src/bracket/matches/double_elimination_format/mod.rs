//! Manage matches from double elimination bracket

mod disqualification;
mod next_opponent;
mod query_state;

use super::{update_bracket_with, Error, Progression};
use crate::{
    bracket::{
        disqualification::get_new_matches,
        progression::{new_matches, winner_of_bracket},
    },
    format::Format,
    matches::{
        double_elimination_matches_from_partition as dem_partition,
        partition_double_elimination_matches, Error as MatchError, Id as MatchId, Match,
        ReportedResult,
    },
    opponent::Opponent,
    player::{Id as PlayerId, Participants, Player},
    seeding::{
        double_elimination_seeded_bracket::get_loser_bracket_matches_top_seed_favored,
        single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
    },
};

/// Computes the next step of a double elimination tournament
#[derive(Clone, Debug)]
pub(crate) struct Step {
    /// True when matches do not need to be validated by the tournament
    /// organiser
    auto: bool,
    /// All matches of double-elimination bracket
    bracket: Vec<Match>,
    /// Seeding used for this bracket
    seeding: Participants,
}

impl Step {
    /// Generate double elimination matches using `seeding`
    ///
    /// # Errors
    /// thrown when math overflow happens
    pub fn new(
        matches: Option<Vec<Match>>,
        seeding: Participants,
        automatic_progression: bool,
    ) -> Result<Self, Error> {
        if let Some(m) = matches {
            Ok(Self {
                seeding,
                bracket: m,
                auto: automatic_progression,
            })
        } else {
            let mut matches = vec![];
            let mut winner_bracket_matches = get_balanced_round_matches_top_seed_favored(&seeding)?;
            matches.append(&mut winner_bracket_matches);
            let mut loser_bracket_matches = get_loser_bracket_matches_top_seed_favored(&seeding)?;
            matches.append(&mut loser_bracket_matches);
            let grand_finals: Match = Match::new([Opponent::Unknown, Opponent::Unknown], [1, 2])?;
            matches.push(grand_finals);
            let grand_finals_reset: Match =
                Match::new([Opponent::Unknown, Opponent::Unknown], [1, 2])?;
            matches.push(grand_finals_reset);

            Ok(Self {
                seeding,
                bracket: matches,
                auto: automatic_progression,
            })
        }
    }

    /// Clear previous reported result for `player_id`
    fn clear_reported_result(self, player_id: PlayerId) -> Result<Self, Error> {
        let match_to_update = self
            .bracket
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
                    .bracket
                    .into_iter()
                    .map(|m| {
                        if m.get_id() == m_to_clear.get_id() {
                            m_to_clear.clone()
                        } else {
                            m
                        }
                    })
                    .collect();
                Ok(Self {
                    bracket: matches,
                    ..self
                })
            }
            None => Err(Error::NoMatchToPlay(player.clone())),
        }
    }
}

/// Place loser from winner's bracket into loser bracket using seed of
/// `expected_loser_seed`. Returns updated loser bracket
fn send_to_losers(
    loser_bracket: &[Match],
    loser: Player,
    expected_loser_seed: usize,
) -> Vec<Match> {
    let loser_match = loser_bracket
        .iter()
        .find(|m| m.is_first_loser_match(expected_loser_seed))
        .expect("match");
    let is_player_1 = expected_loser_seed == loser_match.get_seeds()[0];
    let loser_match = loser_match.clone().set_player(loser, is_player_1);

    update_bracket_with(loser_bracket, &loser_match)
}

/// Update grand finals or reset
fn update_grand_finals_or_reset(
    matches: Vec<Match>,
    match_id: MatchId,
    winner_bracket: Vec<Match>,
    loser_bracket: Vec<Match>,
    gf: Match,
    gf_reset: Match,
) -> Result<Vec<Match>, Error> {
    let mut loser_bracket = loser_bracket;
    if gf.get_id() == match_id {
        let (gf, _, _) = gf.update_outcome()?;
        let mut gf_reset = gf_reset;
        // when a reset happens
        let accept_match_results = gf.get_winner() == gf.get_players()[1];
        if accept_match_results {
            // update grand finals reset match
            if let Opponent::Player(p) = &gf.get_players()[0] {
                gf_reset = gf_reset.set_player(p.clone(), true);
            }
            if let Opponent::Player(p) = &gf.get_players()[1] {
                gf_reset = gf_reset.set_player(p.clone(), false);
            }
        }
        if let Opponent::Player(grand_finals_loser) = gf.get_automatic_loser() {
            if let Opponent::Player(winner_of_winner_bracket) = gf.get_players()[0].clone() {
                if grand_finals_loser == winner_of_winner_bracket {
                    let (update, _, _) = gf_reset
                        .set_automatic_loser(grand_finals_loser.get_id())?
                        .update_outcome()?;
                    gf_reset = update;
                }
            }
        }
        return Ok(dem_partition(&winner_bracket, &loser_bracket, gf, gf_reset));
    } else if gf_reset.get_id() == match_id {
        let (gf_reset, _, _) = gf_reset.update_outcome()?;
        let mut matches = winner_bracket;
        matches.append(&mut loser_bracket);
        matches.push(gf);
        matches.push(gf_reset);
        return Ok(matches);
    }
    Err(Error::NoMatchToUpdate(matches, match_id))
}

impl Progression for Step {
    fn disqualify_participant(
        &self,
        player_id: crate::player::Id,
    ) -> Result<(Vec<Match>, Vec<Match>), Error> {
        if self.is_over() {
            return Err(Error::TournamentIsOver);
        }
        let Some(disqualified) = self.seeding.get(player_id) else {
            return Err(Error::UnknownPlayer(player_id, self.seeding.clone()));
        };

        let old_matches_to_play = self.matches_to_play();

        let Some(m) = self.bracket.iter().rev().find(|m| {
            m.contains(player_id)
                && m.get_winner() == Opponent::Unknown
                && m.get_automatic_loser() == Opponent::Unknown
        }) else {
            let Some(p) = self.seeding.get(player_id) else {
                return Err(Error::UnknownPlayer(player_id, self.seeding.clone()))
            };
           return Err(Error::PlayerDisqualified(p));
        };
        // disqualify player then validate match result to update double elimination bracket
        let current_match_to_play = m.clone().set_automatic_loser(player_id)?;
        let matches = update_bracket_with(&self.bracket, &current_match_to_play);
        let expected_loser_seed = m.get_seeds()[1];
        let (w_bracket, l_bracket, gf, gf_reset) =
            partition_double_elimination_matches(&matches, self.seeding.len())?;
        let l_bracket = send_to_losers(&l_bracket, disqualified, expected_loser_seed);
        let matches = dem_partition(&w_bracket, &l_bracket, gf, gf_reset);
        let p = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
        // move DQ'ed played as far as possible
        match p.validate_match_result(current_match_to_play.get_id()) {
            Ok((bracket, _)) => {
                let Some(m) = bracket
                    .iter()
                    .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown) else {
                    let p = Step::new(Some(bracket.clone()), self.seeding.clone(), self.auto)?;
                    let new_matches = get_new_matches(&old_matches_to_play, &p.matches_to_play());
                    return Ok((bracket, new_matches));
                };
                // DQ them in loser bracket and validate result again
                let match_in_losers = m.clone().set_automatic_loser(player_id)?;
                let bracket = update_bracket_with(&bracket, &match_in_losers);
                let p = Step::new(Some(bracket), self.seeding.clone(), self.auto)?;

                let Ok((bracket,_))= p.validate_match_result(match_in_losers.get_id()) else {
                    let new_matches = get_new_matches(&old_matches_to_play, &p.matches_to_play());
                    return Ok((p.bracket, new_matches));
                };

                let p = Step::new(Some(bracket), self.seeding.clone(), self.auto)?;
                let new_matches = get_new_matches(&old_matches_to_play, &p.matches_to_play());
                Ok((p.bracket, new_matches))
            }
            Err(bracket_e) => {
                // if no winner can be declared because there is a
                // missing player, then don't throw an error
                let Error::MatchUpdate(ref e) = bracket_e else {
                    return Err(bracket_e)
                };
                match e {
                    MatchError::MissingOpponent(_) => {
                        disqualify_player(&p, player_id, &old_matches_to_play)
                    }
                    MatchError::PlayersReportedDifferentMatchOutcome(_, _) => {
                        // Can't update match in losers where where DQ'ed player is in.
                        // Set DQ'ed player as loser and update
                        disqualify_player_and_update_bracket(
                            &p,
                            player_id,
                            &self.seeding,
                            self.auto,
                            &old_matches_to_play,
                        )
                    }
                    _ => Err(bracket_e),
                }
            }
        }
    }

    fn get_format(&self) -> Format {
        Format::DoubleElimination
    }

    fn is_over(&self) -> bool {
        let (winner_bracket, loser_bracket, gf, gfr) =
            partition_double_elimination_matches(&self.bracket, self.seeding.len())
                .expect("partition");
        super::bracket_is_over(&winner_bracket)
            && super::bracket_is_over(&loser_bracket)
            && gf.is_over()
            && (gf.stronger_seed_wins() || gfr.is_over())
    }

    fn matches_to_play(&self) -> Vec<Match> {
        self.bracket
            .iter()
            .cloned()
            .filter(Match::is_playable)
            .collect()
    }

    fn next_opponent(
        &self,
        player_id: crate::player::Id,
    ) -> Result<(Opponent, crate::matches::Id, String), Error> {
        let player = if let Some(p) = self.seeding.get(player_id) {
            p
        } else {
            return Err(Error::PlayerIsNotParticipant(player_id));
        };

        if self.bracket.is_empty() {
            return Err(Error::NoGeneratedMatches);
        }

        if self.is_disqualified(player_id) {
            return Err(Error::Disqualified(player));
        }

        let next_match = self
            .bracket
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let relevant_match = if let Some(m) = next_match {
            m
        } else {
            let last_match = self.bracket.iter().last().expect("last match");
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
            return Err(Error::PlayerIsNotParticipant(player_id));
        };
        if super::is_disqualified(player_id, &self.bracket) {
            return Err(Error::PlayerDisqualified(player));
        }

        let old_matches = self.matches_to_play();
        let Some(m) =self
            .bracket
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown) else
        {
            return Err(Error::NoMatchToPlay(player));
        };
        let affected_match_id = m.get_id();
        let matches = self.clone().update_player_reported_match_result(
            affected_match_id,
            result,
            player_id,
        )?;
        let p = Step::new(Some(matches), self.seeding.clone(), self.auto)?;

        let matches = if self.auto {
            match p.clone().validate_match_result(affected_match_id) {
                Ok((m, _)) => m,
                Err(e) => {
                    if let Error::MatchUpdate(MatchError::PlayersReportedDifferentMatchOutcome(
                        _,
                        _,
                    )) = e
                    {
                        p.bracket
                    } else {
                        return Err(e);
                    }
                }
            }
        } else {
            p.bracket
        };
        let p = Step::new(Some(matches), self.seeding.clone(), self.auto)?;

        let new_matches = new_matches(&old_matches, &p.matches_to_play());
        Ok((p.bracket, affected_match_id, new_matches))
    }

    fn tournament_organiser_reports_result(
        &self,
        player1: crate::player::Id,
        result: (i8, i8),
        player2: crate::player::Id,
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        let result_player_1 = ReportedResult(result);
        let p = self.clone().clear_reported_result(player1)?;
        let p = p.clear_reported_result(player2)?;
        let (matches, first_affected_match, _new_matches) =
            p.report_result(player1, result_player_1.0)?;
        let progression = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
        let (matches, second_affected_match, new_matches) =
            progression.report_result(player2, result_player_1.reverse().0)?;
        assert_eq!(first_affected_match, second_affected_match);
        Ok((matches, first_affected_match, new_matches))
    }

    fn update_player_reported_match_result(
        &self,
        match_id: crate::matches::Id,
        result: (i8, i8),
        player_id: crate::player::Id,
    ) -> Result<Vec<Match>, Error> {
        let m = match self.bracket.iter().find(|m| m.get_id() == match_id) {
            Some(m) => m,
            None => {
                return Err(Error::UnknownMatch(match_id));
            }
        };

        let updated_match = m
            .clone()
            .update_reported_result(player_id, ReportedResult(result))?;
        let matches = self
            .bracket
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

    // NOTE: w_bracket -> winner bracket
    //       l_bracket -> loser bracket
    /// First look if match is in winners, then loosers, then GF, then GF reset
    /// If found in winners, update winners, send loser to losers and update
    /// losers as well
    fn validate_match_result(
        &self,
        match_id: crate::matches::Id,
    ) -> Result<(Vec<Match>, Vec<Match>), Error> {
        let old_matches = self.matches_to_play();
        let (w_bracket, l_bracket, mut gf, mut gf_reset) =
            partition_double_elimination_matches(&self.bracket, self.seeding.len())?;
        let (w_bracket, l_bracket_elements) = match super::update(&w_bracket, match_id) {
            Ok(elements) => elements, // found in winners
            Err(e) => match e {
                Error::UnknownMatch(_bad_winner_match) => {
                    let (l_bracket, _elements) = match super::update(&l_bracket, match_id) {
                        Ok(elements) => elements, // found in losers
                        Err(e) => {
                            if let Error::UnknownMatch(_bad_loser_match) = e {
                                let matches = match update_grand_finals_or_reset(
                                    self.bracket.clone(),
                                    match_id,
                                    w_bracket,
                                    l_bracket,
                                    gf,
                                    gf_reset,
                                ) {
                                    Ok(m) => m,
                                    Err(e) => {
                                        return Err(e);
                                    }
                                }; // if not found in losers either
                                let p = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
                                let new_m = new_matches(&old_matches, &p.matches_to_play());
                                return Ok((p.bracket, new_m));
                            }
                            return Err(e);
                        }
                    };
                    gf = match winner_of_bracket(&l_bracket) {
                        Some(w) => gf.set_opponent(w, false),
                        None => gf,
                    };
                    let matches = dem_partition(&w_bracket, &l_bracket, gf, gf_reset);
                    let p = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
                    let new_matches = new_matches(&old_matches, &p.matches_to_play());
                    return Ok((p.bracket, new_matches));
                }
                _ => {
                    return Err(e);
                }
            },
        };
        let l_bracket = if let Some((loser, expected_loser_seed, is_disqualified_from_winners)) =
            l_bracket_elements
        {
            update_loser_bracket_after_updating_winners_bracket(
                &l_bracket,
                &loser,
                is_disqualified_from_winners,
                expected_loser_seed,
            )
        } else {
            l_bracket
        };
        let winner_of_winner_bracket = winner_of_bracket(&w_bracket);
        if let Some(p) = winner_of_winner_bracket.clone() {
            gf = gf.set_player(p, true);
        }
        let winner_of_loser_bracket = winner_of_bracket(&l_bracket);
        // when loser of winners finals is disqualified, grand finals can be updated
        if let Some(p) = winner_of_loser_bracket {
            gf = gf.set_player(p.clone(), false);
            if w_bracket
                .iter()
                .any(|m| m.is_automatic_loser_by_disqualification(p.clone().get_id()))
            {
                gf = gf.set_automatic_loser(p.get_id())?;
            }
            if gf.needs_update_because_of_disqualified_participant() {
                gf = gf.update_outcome()?.0;
            }
        }
        // when the winner of winner bracket is disqualified, then reset match should be validated also
        if let Opponent::Player(p) = gf.get_automatic_loser() {
            if let Some(winner) = winner_of_winner_bracket {
                if p == winner && gf.is_over() {
                    gf_reset = Match::new(gf.get_players(), [1, 2])?;
                    gf_reset = gf_reset.set_automatic_loser(winner.get_id())?;
                    let (update, _, _) = gf_reset.update_outcome()?;
                    gf_reset = update;
                }
            }
        }
        let matches = dem_partition(&w_bracket, &l_bracket, gf, gf_reset);
        let p = Step::new(Some(matches.clone()), self.seeding.clone(), self.auto)?;
        let new_matches = new_matches(&old_matches, &p.matches_to_play());
        Ok((matches, new_matches))
    }

    fn is_disqualified(&self, player_id: PlayerId) -> bool {
        super::is_disqualified(player_id, &self.bracket)
    }
}

/// Set player as disqualified. Used when there is no need for further updates
fn disqualify_player(
    p: &Step,
    player_id: PlayerId,
    old_matches_to_play: &[Match],
) -> Result<(Vec<Match>, Vec<Match>), Error> {
    // Look in late matches to disqualify player
    let new_matches = get_new_matches(old_matches_to_play, &p.matches_to_play());
    let match_to_set_dq = p
        .bracket
        .iter()
        .rev()
        .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        .expect("match in losers to update")
        .clone()
        .set_automatic_loser(player_id)
        .expect("disqualified player in loser");
    let matches = update_bracket_with(&p.bracket, &match_to_set_dq);
    let p = Step::new(Some(matches), p.seeding.clone(), p.auto)?;
    Ok((p.bracket, new_matches))
}

/// Set player as disqualified and update bracket
fn disqualify_player_and_update_bracket(
    p: &Step,
    player_id: PlayerId,
    seeding: &Participants,
    auto: bool,
    old_bracket: &[Match],
) -> Result<(Vec<Match>, Vec<Match>), Error> {
    let match_to_set_dq = p
        .bracket
        .iter()
        .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        .expect("match in losers to update")
        .clone()
        .set_automatic_loser(player_id)
        .expect("updated match");
    let bracket = update_bracket_with(&p.bracket, &match_to_set_dq);
    let p = Step::new(Some(bracket), seeding.clone(), auto)?;
    let (bracket, _) = p.validate_match_result(match_to_set_dq.get_id())?;
    let p = Step::new(Some(bracket), seeding.clone(), auto)?;
    let new_matches = get_new_matches(old_bracket, &p.matches_to_play());
    Ok((p.bracket, new_matches))
}

/// when disqualifying a player and updating winner bracket, you can then
/// update loser bracket.
///
/// First you send disqualified player to loser, move him if he was not
/// disqualified, then set him as automatic loser in his current loser bracket
/// match.
fn update_loser_bracket_after_updating_winners_bracket(
    l_bracket: &[Match],
    loser: &Player,
    is_disqualified_from_winners: bool,
    expected_loser_seed: usize,
) -> Vec<Match> {
    let l_bracket = send_to_losers(l_bracket, loser.clone(), expected_loser_seed);
    let l_match = l_bracket
        .iter()
        .find(|m| m.contains(loser.get_id()))
        .expect("loser match");
    if is_disqualified_from_winners {
        let l_bracket = match super::update(&l_bracket, l_match.get_id()) {
            Ok((matches, _)) => matches,
            Err(_) => l_bracket.clone(),
        };
        match l_bracket
            .iter()
            .find(|m| m.contains(loser.id) && m.get_winner() == Opponent::Unknown)
        {
            Some(match_to_set_dq) => {
                let match_to_set_dq = match_to_set_dq
                    .clone()
                    .set_automatic_loser(loser.id)
                    .expect("match with disqualified player");
                let l_bracket = update_bracket_with(&l_bracket, &match_to_set_dq);
                match super::update(&l_bracket, l_match.get_id()) {
                    Ok((matches, _)) => matches,
                    Err(_) => l_bracket,
                }
            }
            // loser finishes in GF
            None => l_bracket,
        }
    } else {
        match super::update(&l_bracket.clone(), l_match.get_id()) {
            Ok((l_bracket_matches, _)) => l_bracket_matches,
            Err(_) => l_bracket,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bracket::matches::{double_elimination_format::Step, Progression},
        matches::{partition_double_elimination_matches, Id as MatchId},
        player::{Id as PlayerId, Participants, Player},
    };

    #[test]
    fn run_3_man_bracket() {
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let automatic_progression = true;
        let p = Step::new(None, seeding.clone(), automatic_progression).expect("progression");

        assert_eq!(p.bracket.len(), 5);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding, automatic_progression).expect("progression");
        assert!(p.is_over());
    }

    #[test]
    fn partition_matches_for_3_man_bracket() {
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let automatic_progression = true;
        let p = Step::new(None, seeding, automatic_progression).expect("progression");

        let (winner_bracket, loser_bracket, _gf, _gfr) =
            partition_double_elimination_matches(&p.bracket, p.seeding.len()).expect("partition");
        assert_eq!(winner_bracket.len(), 2);
        assert_eq!(loser_bracket.len(), 1);
        assert_eq!(loser_bracket[0].get_seeds(), [2, 3]);
    }

    #[test]
    fn run_5_man_bracket() {
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut seeding = Participants::default();
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let automatic_progression = true;
        let p = Step::new(None, seeding.clone(), automatic_progression).expect("progression");

        assert_eq!(p.bracket.len(), 9);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[4])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[5])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[1])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding, automatic_progression).expect("progression");
        assert!(p.is_over());
    }

    #[test]
    fn run_8_man_bracket_no_upsets() {
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let automatic_progression = true;
        let p = Step::new(None, seeding.clone(), automatic_progression).expect("progression");

        assert_eq!(p.bracket.len(), 15);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[8])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[7])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[8])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[6], (2, 0), player_ids[7])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[4])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let p =
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding, automatic_progression).expect("progression");
        assert!(p.is_over());
    }

    fn report(
        p: &Step,
        player1: PlayerId,
        result: (i8, i8),
        player2: PlayerId,
        seeding: &Participants,
        automatic_progression: bool,
    ) -> (Step, MatchId) {
        let (matches, m_id, _new_matches) = p
            .tournament_organiser_reports_result(player1, result, player2)
            .expect("bracket");
        (
            Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression"),
            m_id,
        )
    }

    fn validate(
        p: &Step,
        match_id: MatchId,
        seeding: &Participants,
        automatic_progression: bool,
    ) -> Step {
        let (matches, _new_matches) = p.validate_match_result(match_id).expect("bracket");
        Step::new(Some(matches), seeding.clone(), automatic_progression).expect("progression")
    }

    #[test]
    fn run_8_man_bracket_with_frequent_upsets() {
        // every 2 matches, there is an upset
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let auto = false;
        let p = Step::new(None, seeding.clone(), auto).expect("progression");
        assert_eq!(p.bracket.len(), 15);
        let (p, winner_1vs8) = report(&p, player_ids[1], (2, 0), player_ids[8], &seeding, auto);
        let p = validate(&p, winner_1vs8, &seeding, auto);
        let (p, winner_2vs7) = report(&p, player_ids[2], (0, 2), player_ids[7], &seeding, auto);
        let p = validate(&p, winner_2vs7, &seeding, auto);
        let (p, winner_3vs6) = report(&p, player_ids[3], (2, 0), player_ids[6], &seeding, auto);
        let p = validate(&p, winner_3vs6, &seeding, auto);
        let (p, winner_4vs5) = report(&p, player_ids[4], (0, 2), player_ids[5], &seeding, auto);
        let p = validate(&p, winner_4vs5, &seeding, auto);
        let (p, loser_4vs8) = report(&p, player_ids[4], (2, 0), player_ids[8], &seeding, auto);
        let p = validate(&p, loser_4vs8, &seeding, auto);
        let p = Step::new(Some(p.bracket), seeding.clone(), auto).expect("progression");
        let (matches, loser_2vs6, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[6])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_2vs6).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, winner_1vs5, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(winner_1vs5).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, winner_3vs7, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (0, 2), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(winner_3vs7).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, loser_3vs6, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_3vs6).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, loser_4vs5, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_4vs5).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, loser_3vs5, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_3vs5).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, winner_1vs7, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(winner_1vs7).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, loser_1vs3, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_1vs3).expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, grand_finals, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), seeding.clone(), auto).expect("progression");
        let (matches, _) = p.validate_match_result(grand_finals).expect("bracket");
        let p = Step::new(Some(matches), seeding, auto).expect("progression");
        assert!(p.is_over(), "{p:?}");
    }
}
