//! Manage matches from double elimination bracket

mod disqualification;
mod next_opponent;
mod query_state;

use super::{
    assert_disqualified_at_most_once, assert_match_is_well_formed, update_bracket_with, Error,
    Progression,
};
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
    player::Id as PlayerId,
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
    matches: Vec<Match>,
    /// Seeding used for this bracket
    seeding: Vec<PlayerId>,
}

impl Step {
    /// Generate double elimination matches using `seeding`
    ///
    /// # Errors
    /// thrown when math overflow happens
    pub fn new(
        matches: Option<Vec<Match>>,
        seeding: Vec<PlayerId>,
        automatic_progression: bool,
    ) -> Result<Self, Error> {
        let Some(matches) = matches else {
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

            return Ok(Self {
                seeding,
                matches,
                auto: automatic_progression,
            })
        };
        Ok(Self {
            seeding,
            matches,
            auto: automatic_progression,
        })
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
                let matches = update_bracket_with(&self.matches, &m_to_clear);
                Ok(Self { matches, ..self })
            }
            None => Err(Error::NoMatchToPlay(player_id)),
        }
    }
}

/// Place loser from winner's bracket into loser bracket using seed of
/// `expected_loser_seed`. Returns updated loser bracket
fn send_to_losers(
    loser_bracket: &[Match],
    loser: PlayerId,
    expected_loser_seed: usize,
) -> Result<Vec<Match>, Error> {
    let loser_match = loser_bracket
        .iter()
        .find(|m| m.is_first_loser_match(expected_loser_seed))
        .expect("match");
    let is_player_1 = expected_loser_seed == loser_match.get_seeds()[0];
    let loser_match = (*loser_match).insert_player(loser, is_player_1)?;

    Ok(update_bracket_with(loser_bracket, &loser_match))
}

/// Update grand finals or reset
fn update_grand_finals_or_reset(
    match_id: MatchId,
    winner_bracket: Vec<Match>,
    loser_bracket: Vec<Match>,
    gf: Match,
    gf_reset: Match,
) -> Result<Vec<Match>, Error> {
    match match_id {
        id if id == gf.get_id() => {
            let (gf, _, _) = gf.update_outcome()?;
            // when a reset happens in grand finals
            let gf_reset = match (gf.get_winner(), gf.get_players()[1]) {
                (Opponent::Player(gf_winner), Opponent::Player(player_from_losers))
                    if gf_winner == player_from_losers =>
                {
                    // Set players of gf reset
                    let gf_reset = match gf.get_players() {
                        [Opponent::Player(p1), Opponent::Player(p2)] => {
                            let ggf_reset = gf_reset.insert_player(p1, true)?;
                            ggf_reset.insert_player(p2, false)?
                        }
                        [Opponent::Player(p), _] => gf_reset.insert_player(p, true)?,
                        [_, Opponent::Player(p)] => gf_reset.insert_player(p, false)?,
                        _ => gf_reset,
                    };

                    // if player is disqualified in grand finals, update gf reset
                    match (gf.get_automatic_loser(), gf.get_players()[0]) {
                        (
                            Opponent::Player(grand_finals_loser),
                            Opponent::Player(winner_of_winner_bracket),
                        ) if grand_finals_loser == winner_of_winner_bracket => {
                            gf_reset
                                .set_automatic_loser(grand_finals_loser)?
                                .update_outcome()?
                                .0
                        }
                        (_, _) => gf_reset,
                    }
                }
                _ => gf_reset,
            };

            Ok(dem_partition(&winner_bracket, &loser_bracket, gf, gf_reset))
        }
        id if id == gf_reset.get_id() => {
            let (gf_reset, _, _) = gf_reset.update_outcome()?;
            Ok([winner_bracket, loser_bracket, vec![gf, gf_reset]].concat())
        }
        _ => panic!("expected GF or GF reset but got other match: {match_id}"),
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
        if !self.seeding.contains(&player_id) {
            return Err(Error::UnknownPlayer(player_id, self.seeding.clone()));
        };
        let disqualified = player_id;

        let old_matches_to_play = self.matches_to_play();

        let Some(m) = self.matches.iter().rev().find(|m| {
            m.contains(player_id)
                && m.get_winner() == Opponent::Unknown
                && m.get_automatic_loser() == Opponent::Unknown
        }) else {
            if !self.seeding.contains(&player_id) {
                return Err(Error::UnknownPlayer(player_id, self.seeding.clone()))
            };
           return Err(Error::ForbiddenDisqualified(player_id));
        };
        // disqualify player then validate match result to update double elimination bracket
        let current_match_to_play = (*m).set_automatic_loser(player_id)?;
        let matches = update_bracket_with(&self.matches, &current_match_to_play);
        let expected_loser_seed = m.get_seeds()[1];
        let (w_bracket, l_bracket, gf, gf_reset) =
            partition_double_elimination_matches(&matches, self.seeding.len());
        // don't send to loser if the disqualified player is in gf or gf_reset
        let l_bracket = if gf.contains(disqualified)
            || gf.contains(disqualified)
            || l_bracket.iter().any(|m| m.contains(disqualified))
        {
            l_bracket
        } else {
            send_to_losers(&l_bracket, disqualified, expected_loser_seed)?
        };
        let matches = dem_partition(&w_bracket, &l_bracket, gf, gf_reset);
        let p = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
        // move DQ'ed played as far as possible
        match p.validate_match_result(current_match_to_play.get_id()) {
            Ok((bracket, _)) => {
                let Some(m) = bracket
                    .iter()
                    .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown) else {
                    let p = Step::new(Some(bracket.clone()), self.seeding.clone(),self.auto)?;
                    let new_matches = get_new_matches(&old_matches_to_play, &p.matches_to_play());
                    return Ok((bracket, new_matches));
                };
                // DQ them in loser bracket and validate result again
                let match_in_losers = (*m).set_automatic_loser(player_id)?;
                let bracket = update_bracket_with(&bracket, &match_in_losers);
                let p = Step::new(Some(bracket), self.seeding.clone(), self.auto)?;

                let Ok((bracket,_))= p.validate_match_result(match_in_losers.get_id()) else {
                    let new_matches = get_new_matches(&old_matches_to_play, &p.matches_to_play());
                    return Ok((p.matches, new_matches));
                };

                let p = Step::new(Some(bracket), self.seeding.clone(), self.auto)?;
                let new_matches = get_new_matches(&old_matches_to_play, &p.matches_to_play());
                Ok((p.matches, new_matches))
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
            partition_double_elimination_matches(&self.matches, self.seeding.len());
        let Some(stronger_seed_wins) = gf.stronger_seed_wins() else {
            return false;
        };
        super::bracket_is_over(&winner_bracket)
            && super::bracket_is_over(&loser_bracket)
            && gf.is_over()
            && (stronger_seed_wins || gfr.is_over())
    }

    fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .iter()
            .copied()
            .filter(Match::needs_playing)
            .collect()
    }

    fn next_opponent(
        &self,
        player_id: crate::player::Id,
    ) -> Result<(Opponent, crate::matches::Id), Error> {
        if !self.seeding.contains(&player_id) {
            return Err(Error::PlayerIsNotParticipant(player_id));
        };

        if self.matches.is_empty() {
            return Err(Error::NoGeneratedMatches);
        }

        if self.is_disqualified(player_id) {
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
            }
        };

        let opponent = match relevant_match.get_players() {
            [Opponent::Player(p1), Opponent::Player(p2)] if p1 == player_id => Opponent::Player(p2),
            [Opponent::Player(p1), Opponent::Player(p2)] if p2 == player_id => Opponent::Player(p1),
            _ => Opponent::Unknown,
        };

        Ok((opponent, relevant_match.get_id()))
    }

    fn report_result(
        &self,
        player_id: crate::player::Id,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        if !self.seeding.contains(&player_id) {
            return Err(Error::PlayerIsNotParticipant(player_id));
        };
        if super::is_disqualified(player_id, &self.matches) {
            return Err(Error::ForbiddenDisqualified(player_id));
        }

        let old_matches = self.matches_to_play();
        let Some(m) = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown) else
        {
            return Err(Error::NoMatchToPlay(player_id));
        };
        let affected_match_id = m.get_id();
        let matches = self.clone().update_player_reported_match_result(
            affected_match_id,
            result,
            player_id,
        )?;
        let bracket = Step::new(Some(matches), self.seeding.clone(), self.auto)?;

        let matches = if self.auto {
            match bracket.clone().validate_match_result(affected_match_id) {
                Ok((m, _)) => m,
                Err(Error::MatchUpdate(MatchError::PlayersReportedDifferentMatchOutcome(_, _))) => {
                    bracket.matches
                }
                Err(e) => return Err(e),
            }
        } else {
            bracket.matches
        };
        let p = Step::new(Some(matches), self.seeding.clone(), self.auto)?;

        let new_matches = new_matches(&old_matches, &p.matches_to_play());
        Ok((p.matches, affected_match_id, new_matches))
    }

    fn tournament_organiser_reports_result(
        &self,
        player1: crate::player::Id,
        result: (i8, i8),
        player2: crate::player::Id,
    ) -> Result<(Vec<Match>, crate::matches::Id, Vec<Match>), Error> {
        // clear reported results
        let bracket = self.clone().clear_reported_result(player1)?;
        let bracket = bracket.clear_reported_result(player2)?;

        // report score as p1
        let result_player_1 = ReportedResult(result);
        let (matches, first_affected_match, _new_matches) =
            bracket.report_result(player1, result_player_1.0)?;

        // report same score as p2
        let bracket = Step::new(Some(matches), self.seeding.clone(), self.auto)?;

        let (matches, second_affected_match, new_matches) =
            bracket.report_result(player2, result_player_1.reverse().0)?;

        assert_eq!(first_affected_match, second_affected_match);

        Ok((matches, first_affected_match, new_matches))
    }

    fn update_player_reported_match_result(
        &self,
        match_id: crate::matches::Id,
        result: (i8, i8),
        player_id: crate::player::Id,
    ) -> Result<Vec<Match>, Error> {
        let Some(m) = self.matches.iter().find(|m| m.get_id() == match_id) else {
            return Err(Error::UnknownMatch(match_id));
        };

        let updated_match = (*m).update_reported_result(player_id, ReportedResult(result))?;
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
        let (w_bracket, l_bracket, gf, gf_reset) =
            partition_double_elimination_matches(&self.matches, self.seeding.len());
        match super::update(&w_bracket, match_id) {
            Ok((w_bracket, l_bracket_elements)) => {
                let l_bracket = match l_bracket_elements {
                    Some((loser, expected_loser_seed, is_disqualified_from_winners)) => {
                        update_loser_bracket_after_updating_winners_bracket(
                            &l_bracket,
                            loser,
                            is_disqualified_from_winners,
                            expected_loser_seed,
                        )?
                    }
                    None => l_bracket,
                };

                let gf = match winner_of_bracket(&w_bracket) {
                    Some(winner_of_winner_bracket) => {
                        gf.insert_player(winner_of_winner_bracket, true)?
                    }
                    None => gf,
                };
                // when loser of winners finals is disqualified, grand finals can be updated
                let gf = match winner_of_bracket(&l_bracket) {
                    Some(winner_of_loser_bracket) => {
                        let gf = gf.insert_player(winner_of_loser_bracket, false)?;

                        if w_bracket.iter().any(|m| {
                            m.is_automatic_loser_by_disqualification(winner_of_loser_bracket)
                        }) {
                            gf.set_automatic_loser(winner_of_loser_bracket)?
                                .update_outcome()?
                                .0
                        } else {
                            gf
                        }
                    }
                    None => gf,
                };
                // when the winner of winner bracket is disqualified, then reset match should be validated also
                let gf_reset = match (
                    gf.get_automatic_loser(),
                    winner_of_bracket(&w_bracket),
                    gf.is_over(),
                ) {
                    (Opponent::Player(disqualified), Some(winner_of_winner_bracket), true)
                        if disqualified == winner_of_winner_bracket =>
                    {
                        Match::new(gf.get_players(), [1, 2])?
                            .set_automatic_loser(winner_of_winner_bracket)?
                            .update_outcome()?
                            .0
                    }
                    _ => gf_reset,
                };

                let matches = dem_partition(&w_bracket, &l_bracket, gf, gf_reset);
                let bracket = Step::new(Some(matches.clone()), self.seeding.clone(), self.auto)?;
                let new_matches = new_matches(&old_matches, &bracket.matches_to_play());
                Ok((matches, new_matches))
            }
            Err(Error::UnknownMatch(_bad_winner_match)) => {
                match super::update(&l_bracket, match_id) {
                    Ok((l_bracket, _elements)) => {
                        // send winner of loser bracket to grand finals if
                        // possible
                        let gf = match winner_of_bracket(&l_bracket) {
                            Some(winner_of_loser_bracket) => {
                                gf.set_player(winner_of_loser_bracket, false)
                            }
                            None => gf,
                        };
                        let matches = match (gf.get_players(), gf.get_automatic_loser()) {
                            ([Opponent::Player(_), Opponent::Player(_)], Opponent::Player(_)) => {
                                update_grand_finals_or_reset(
                                    gf.get_id(),
                                    w_bracket,
                                    l_bracket,
                                    gf,
                                    gf_reset,
                                )
                                .expect("grand finals updated")
                            }
                            _ => dem_partition(&w_bracket, &l_bracket, gf, gf_reset),
                        };
                        let bracket = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
                        let new_matches = new_matches(&old_matches, &bracket.matches_to_play());
                        Ok((bracket.matches, new_matches))
                    }
                    Err(Error::UnknownMatch(_bad_loser_match)) => {
                        let matches = update_grand_finals_or_reset(
                            match_id, w_bracket, l_bracket, gf, gf_reset,
                        )?;
                        let bracket = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
                        let new_m = new_matches(&old_matches, &bracket.matches_to_play());
                        Ok((bracket.matches, new_m))
                    }
                    Err(e) => Err(e),
                }
            }
            Err(e) => Err(e),
        }
    }

    fn is_disqualified(&self, player_id: PlayerId) -> bool {
        super::is_disqualified(player_id, &self.matches)
    }

    fn check_all_assertions(&self) {
        let (w_bracket, l_bracket, _gf, _gf_reset) =
            partition_double_elimination_matches(&self.matches, self.seeding.len());
        assert_disqualified_at_most_once(&w_bracket, &self.seeding);
        assert_disqualified_at_most_once(&l_bracket, &self.seeding);
        for m in &self.matches {
            assert_match_is_well_formed(m);
        }
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
    let match_to_set_dq = (*p
        .matches
        .iter()
        .rev()
        .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        .expect("match in losers to update"))
    .set_automatic_loser(player_id)
    .expect("disqualified player in loser");
    let matches = update_bracket_with(&p.matches, &match_to_set_dq);
    let p = Step::new(Some(matches), p.seeding.clone(), p.auto)?;
    Ok((p.matches, new_matches))
}

/// Set player as disqualified and update bracket
fn disqualify_player_and_update_bracket(
    p: &Step,
    player_id: PlayerId,
    seeding: &[PlayerId],
    auto: bool,
    old_bracket: &[Match],
) -> Result<(Vec<Match>, Vec<Match>), Error> {
    let match_to_set_dq = (*p
        .matches
        .iter()
        .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        .expect("match in losers to update"))
    .set_automatic_loser(player_id)
    .expect("updated match");
    let bracket = update_bracket_with(&p.matches, &match_to_set_dq);
    let p = Step::new(Some(bracket), seeding.to_vec(), auto)?;
    let (bracket, _) = p.validate_match_result(match_to_set_dq.get_id())?;
    let p = Step::new(Some(bracket), p.seeding, auto)?;
    let new_matches = get_new_matches(old_bracket, &p.matches_to_play());
    Ok((p.matches, new_matches))
}

/// when disqualifying a player and updating winner bracket, you can then
/// update loser bracket.
///
/// First you send disqualified player to loser, move him if he was not
/// disqualified, then set him as automatic loser in his current loser bracket
/// match.
fn update_loser_bracket_after_updating_winners_bracket(
    l_bracket: &[Match],
    loser: PlayerId,
    is_disqualified_from_winners: bool,
    expected_loser_seed: usize,
) -> Result<Vec<Match>, Error> {
    let l_bracket = send_to_losers(l_bracket, loser, expected_loser_seed)?;
    let l_match = l_bracket
        .iter()
        .find(|m| m.contains(loser))
        .expect("loser match");
    if is_disqualified_from_winners {
        let l_bracket = match super::update(&l_bracket, l_match.get_id()) {
            Ok((matches, _)) => matches,
            Err(_) => l_bracket.clone(),
        };
        let l_bracket = match l_bracket
            .iter()
            .find(|m| m.contains(loser) && m.get_winner() == Opponent::Unknown)
        {
            Some(match_to_set_dq) => {
                let match_to_set_dq = (*match_to_set_dq)
                    .set_automatic_loser(loser)
                    .expect("match with disqualified player");
                let l_bracket = update_bracket_with(&l_bracket, &match_to_set_dq);
                match super::update(&l_bracket, l_match.get_id()) {
                    Ok((matches, _)) => matches,
                    Err(_) => l_bracket,
                }
            }
            // loser finishes in GF
            None => l_bracket,
        };
        Ok(l_bracket)
    } else {
        let l_bracket = match super::update(&l_bracket.clone(), l_match.get_id()) {
            Ok((l_bracket_matches, _)) => l_bracket_matches,
            Err(_) => l_bracket,
        };
        Ok(l_bracket)
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
        let mut unpadded_player_ids = vec![]; // padding for readability
        let mut seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            unpadded_player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let auto = true;
        let p = Step::new(None, unpadded_player_ids, auto).expect("progression");

        assert_eq!(p.matches.len(), 5);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        assert!(p.is_over());
    }

    #[test]
    fn partition_matches_for_3_man_bracket() {
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut unpadded_player_ids = vec![];
        let mut seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            unpadded_player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let auto = true;
        let p = Step::new(None, unpadded_player_ids, auto).expect("progression");

        let (winner_bracket, loser_bracket, _gf, _gfr) =
            partition_double_elimination_matches(&p.matches, p.seeding.len());
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
        let p = Step::new(
            None,
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            automatic_progression,
        )
        .expect("progression");

        assert_eq!(p.matches.len(), 9);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[4])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[1])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, automatic_progression).expect("progression");
        assert!(p.is_over());
    }

    #[test]
    fn run_8_man_bracket_no_upsets() {
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut unpadded_player_ids = vec![];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            unpadded_player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let auto = true;
        let p = Step::new(None, unpadded_player_ids, auto).expect("progression");

        assert_eq!(p.matches.len(), 15);
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[8])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[8])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[6], (2, 0), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[4])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        assert!(p.is_over());
    }

    fn report(
        p: &Step,
        player1: PlayerId,
        result: (i8, i8),
        player2: PlayerId,
        automatic_progression: bool,
    ) -> (Step, MatchId) {
        let (matches, m_id, _new_matches) = p
            .tournament_organiser_reports_result(player1, result, player2)
            .expect("bracket");
        (
            Step::new(Some(matches), p.seeding.clone(), automatic_progression)
                .expect("progression"),
            m_id,
        )
    }

    fn validate(p: &Step, match_id: MatchId, automatic_progression: bool) -> Step {
        let (matches, _new_matches) = p.validate_match_result(match_id).expect("bracket");
        Step::new(Some(matches), p.seeding.clone(), automatic_progression).expect("progression")
    }

    #[test]
    fn run_8_man_bracket_with_frequent_upsets() {
        // every 2 matches, there is an upset
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        let mut unpadded_player_ids = vec![]; // padding for readability
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            unpadded_player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("new participant");
        }
        let auto = false;
        let p = Step::new(None, unpadded_player_ids, auto).expect("progression");
        assert_eq!(p.matches.len(), 15);
        let (p, winner_1vs8) = report(&p, player_ids[1], (2, 0), player_ids[8], auto);
        let p = validate(&p, winner_1vs8, auto);
        let (p, winner_2vs7) = report(&p, player_ids[2], (0, 2), player_ids[7], auto);
        let p = validate(&p, winner_2vs7, auto);
        let (p, winner_3vs6) = report(&p, player_ids[3], (2, 0), player_ids[6], auto);
        let p = validate(&p, winner_3vs6, auto);
        let (p, winner_4vs5) = report(&p, player_ids[4], (0, 2), player_ids[5], auto);
        let p = validate(&p, winner_4vs5, auto);
        let (p, loser_4vs8) = report(&p, player_ids[4], (2, 0), player_ids[8], auto);
        let p = validate(&p, loser_4vs8, auto);
        let p = Step::new(Some(p.matches), p.seeding, auto).expect("progression");
        let (matches, loser_2vs6, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[6])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_2vs6).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, winner_1vs5, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(winner_1vs5).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, winner_3vs7, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (0, 2), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(winner_3vs7).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, loser_3vs6, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_3vs6).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, loser_4vs5, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_4vs5).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, loser_3vs5, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[5])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_3vs5).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, winner_1vs7, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(winner_1vs7).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, loser_1vs3, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(loser_1vs3).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, grand_finals, _new_matches) = p
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
            .expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        let (matches, _) = p.validate_match_result(grand_finals).expect("bracket");
        let p = Step::new(Some(matches), p.seeding, auto).expect("progression");
        assert!(p.is_over(), "{p:?}");
    }
}
