//! Progression of a double elimination bracket

use crate::bracket::matches::{update_bracket_with, Error};
use crate::bracket::progression::{new_matches_to_play_for_bracket, winner_of_bracket};
use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::{
    double_elimination_matches_from_partition, partition_double_elimination_matches, Id, Match,
    ReportedResult,
};
use crate::opponent::Opponent;
use crate::validation::AutomaticMatchValidationMode;
use crate::ID;

/// Error while reporting for a double elimination bracket
#[derive(Debug)]
pub enum DoubleEliminationReportResultError {
    /// Player is disqualified
    ///
    /// Player ID is valid but disqualified player are not allowed to report
    // FIXME add test When a player is DQ'd for the match, he cannot report that match
    ForbiddenDisqualified(ID),
    /// No match to play for player
    ///
    /// May happen if tournament organiser validated right before player did for the same match
    // FIXME add test where player has won grand finals but him reporting results in a message that
    //  they won
    NoMatchToPlay(ID),
    /// Match result was reported and validated already.
    // FIXME add test where reporting twice for match results in error
    ResultValidatedAlready,
}

/// All methods to update matches of an ongoing double elimination bracket
pub trait ProgressionDEB {
    /// Tournament organiser reports result for a given match
    ///
    /// # Errors
    /// FIXME add test Reporting twice the same results
    fn tournament_organiser_reports_result(
        self,
        match_id: ID,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>;

    /// Tournament organiser reports result
    ///
    /// NOTE: both players are needed, so it is less ambiguous when reading code:
    /// * p1 2-0 is more ambiguous to read than
    /// * p1 2-0 p2
    ///
    /// Technically, it's unnecessary.
    ///
    /// This method is dangerous because in a double-elimination bracket, it's
    /// possible that a player plays against the same opponent twice, like grand
    /// finals into grand final reset. If a request to update grand finals is
    /// sent twice by accident, then grand finals AND grand final reset match
    /// may get updated. While this is a niche corner case, you may want to use
    /// the safer method `tournament_organiser_reports_result`
    ///
    /// # Panics
    /// When either `player1` or `player2` is unknown
    /// # Error
    /// FIXME add test Reporting result for people that are not playing each other
    fn tournament_organiser_reports_result_dangerous(
        self,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>;

    /// Tournament organiser reports `result` for match where `player` is involved.
    ///
    /// Example: player says "I won 2-0" or "I lost 0-2, but it was close though"
    ///
    /// This method is dangerous because not idempotent
    ///
    /// # Panics
    /// FIXME add test When player is unknown
    /// # Error
    /// FIXME add test When player has played all their matches (won/eliminated)
    /// FIXME use struct BracketResult (Unsigned integer x2)
    fn tournament_organiser_reports_result_for_single_player_dangerous(
        self,
        player: ID,
        result: (i8, i8),
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>;

    /// Report result of player.
    ///
    /// This method is dangerous because it is not idempotent: if you sent the result twice when you
    /// meant to send it once, you can accidentally update two matches.
    ///
    /// # Panics
    /// FIXME add test When player is unknown
    /// FIXME use struct BracketResult (Unsigned integer x2)
    fn report_result_dangerous(
        self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, ID, Vec<Match>), DoubleEliminationReportResultError>;

    /// Update `match_id` with reported `result` of `player`
    ///
    /// # Panics
    /// * FIXME add test When `match_id` is unknown
    /// * FIXME add test When `player_id` is unknown
    /// FIXME use struct BracketResult (Unsigned integer x2)
    /// FIXME add test (0, 0)
    fn update_player_reported_match_result(
        self,
        match_id: ID,
        result: (i8, i8),
        player_id: ID,
    ) -> Self;

    // FIXME doc
    // FIXME determine if it's better to assert or return an error
    //  IMO you should inspect match state before validating. You should try to
    //  validate match always by default when automatic validation is on.
    /// Update bracket with a new match result
    ///
    /// Returns updated bracket and new matches to play. Uses `match_id` as the
    /// first match to start updating before looking deeper into the bracket
    ///
    /// First look if match is in winners, then losers, then GF, then GF reset
    /// If found in winners, update winners, send loser to losers and update
    /// losers as well
    ///
    /// # Panics
    /// When `match_id` is invalid
    ///
    /// # Error
    /// * FIXME add test When `match_id` is unknown
    /// * FIXME add test When validating `match_id` is not possible
    fn validate_match_result(self, match_id: ID) -> (DoubleEliminationBracket, Vec<Match>);

    /// List all matches that can be played out
    fn matches_to_play(&self) -> Vec<Match>;
}

impl ProgressionDEB for DoubleEliminationBracket {
    fn tournament_organiser_reports_result(
        self,
        match_id: ID,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>
    {
        todo!()
    }

    fn tournament_organiser_reports_result_dangerous(
        self,
        player1: ID,
        result: (i8, i8),
        player2: ID,
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>
    {
        assert!(
            self.seeding.contains(player1),
            "{player1} does not belong in bracket"
        );
        assert!(
            self.seeding.contains(player2),
            "{player2} does not belong in bracket"
        );
        // clear reported results
        let bracket = self.clone().clear_reported_result(player1);
        let bracket = bracket.clear_reported_result(player2);

        let matches_where_player1_is_playing: Vec<Match> = bracket
            .matches
            .clone()
            .into_iter()
            .filter(|m| m.contains(player1) && !m.is_over())
            .collect();
        assert!(
            matches_where_player1_is_playing.len() <= 1,
            "player 1 {player1} is involved in only 1 match but they are involved in {:?}",
            matches_where_player1_is_playing
        );
        let matches_where_player2_is_playing: Vec<Match> = bracket
            .clone()
            .matches
            .into_iter()
            .filter(|m| m.contains(player2) && !m.is_over())
            .collect();
        assert!(
            matches_where_player2_is_playing.len() <= 1,
            "player 2 {player2} is involved in only 1 match but they are involved in {:?}",
            matches_where_player2_is_playing
        );

        // report score as p1
        let result_player_1 = ReportedResult(Some(result));
        let (matches, first_affected_match, _new_matches) =
            bracket.report_result_dangerous(player1, result_player_1.0.expect("result"))?;

        // // report same score as p2
        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        let (matches, second_affected_match, new_matches) = bracket
            .report_result_dangerous(player2, result_player_1.reverse().0.expect("result"))?;
        //
        // assert_eq!(first_affected_match, second_affected_match);
        //
        // Ok((matches, first_affected_match, new_matches))
        todo!()
    }

    fn tournament_organiser_reports_result_for_single_player_dangerous(
        self,
        player_left: ID,
        result: (i8, i8),
    ) -> Result<(DoubleEliminationBracket, Id, Vec<Match>), DoubleEliminationReportResultError>
    {
        todo!()
    }

    fn report_result_dangerous(
        self,
        player_id: ID,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, ID, Vec<Match>), DoubleEliminationReportResultError> {
        assert!(self.seeding.contains(player_id));
        if crate::bracket::matches::is_disqualified(player_id, &self.matches) {
            return Err(DoubleEliminationReportResultError::ForbiddenDisqualified(
                player_id,
            ));
        }

        let old_matches = self.matches.clone();
        let Some(m) = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        else {
            return Err(DoubleEliminationReportResultError::NoMatchToPlay(player_id));
        };
        let affected_match_id = m.get_id();
        let bracket =
            self.update_player_reported_match_result(affected_match_id, result, player_id);

        let bracket =
            if bracket.automatic_match_validation_mode == AutomaticMatchValidationMode::Strict {
                bracket
            } else {
                if let Some(match_to_validate) =
                    bracket.matches.iter().find(|m| m.id == affected_match_id)
                {
                    if match_to_validate.has_all_player_reports() {
                        bracket.validate_match_result(affected_match_id).0
                    } else {
                        bracket
                    }
                } else {
                    panic!()
                }
            };
        // // println!("{:?}", old_matches);
        // // println!("{:?}", p.matches_to_play());
        let new_matches = new_matches_to_play_for_bracket(&old_matches, &bracket.matches_to_play());
        Ok((bracket.matches, affected_match_id, new_matches))
    }

    fn update_player_reported_match_result(
        self,
        match_id: ID,
        result: (i8, i8),
        player_id: ID,
    ) -> Self {
        let Some(m) = self.matches.iter().find(|m| m.get_id() == match_id) else {
            panic!("unknown match")
        };
        assert!(self.seeding.contains(player_id));

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
        Self { matches, ..self }
    }

    fn validate_match_result(self, match_id: ID) -> (DoubleEliminationBracket, Vec<Match>) {
        // NOTE: w_bracket -> winner bracket
        //       l_bracket -> loser bracket
        let old_matches = self.matches.clone();
        let (w_bracket, l_bracket, gf, gf_reset) =
            partition_double_elimination_matches(&self.matches, self.seeding.len());
        match crate::bracket::matches::update(&w_bracket, match_id) {
            Ok((w_bracket, l_bracket_elements)) => {
                let l_bracket = match l_bracket_elements {
                    Some((loser, expected_loser_seed, is_disqualified_from_winners)) => {
                        update_loser_bracket_after_updating_winners_bracket(
                            &l_bracket,
                            loser,
                            is_disqualified_from_winners,
                            expected_loser_seed,
                        )
                    }
                    None => l_bracket,
                };

                let gf = match winner_of_bracket(&w_bracket) {
                    Some(winner_of_winner_bracket) => {
                        gf.insert_player(winner_of_winner_bracket, true)
                    }
                    None => gf,
                };
                // when loser of winners finals is disqualified, grand finals can be updated
                let gf = match winner_of_bracket(&l_bracket) {
                    Some(winner_of_loser_bracket) => {
                        let gf = gf.insert_player(winner_of_loser_bracket, false);

                        if w_bracket.iter().any(|m| {
                            m.is_automatic_loser_by_disqualification(winner_of_loser_bracket)
                        }) {
                            gf.set_automatic_loser(winner_of_loser_bracket)
                                .update_outcome()
                                .unwrap()
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
                        Match::new(gf.get_players(), [1, 2])
                            .expect("grand final reset")
                            .set_automatic_loser(winner_of_winner_bracket)
                            .update_outcome()
                            .unwrap()
                            .0
                    }
                    _ => gf_reset,
                };

                let matches =
                    double_elimination_matches_from_partition(&w_bracket, &l_bracket, gf, gf_reset);
                let bracket = DoubleEliminationBracket::new(
                    matches,
                    self.seeding,
                    self.automatic_match_validation_mode,
                );
                let new_matches =
                    new_matches_to_play_for_bracket(&old_matches, &bracket.matches_to_play());
                (bracket, new_matches)
            }
            Err(Error::UnknownMatch(_bad_winner_match)) => {
                //         match super::update(&l_bracket, match_id) {
                //             Ok((l_bracket, _elements)) => {
                //                 // send winner of loser bracket to grand finals if
                //                 // possible
                //                 let gf = match winner_of_bracket(&l_bracket) {
                //                     Some(winner_of_loser_bracket) => {
                //                         gf.set_player(winner_of_loser_bracket, false)
                //                     }
                //                     None => gf,
                //                 };
                //                 let matches = match (gf.get_players(), gf.get_automatic_loser()) {
                //                     ([Opponent::Player(_), Opponent::Player(_)], Opponent::Player(_)) => {
                //                         update_grand_finals_or_reset(
                //                             gf.get_id(),
                //                             w_bracket,
                //                             l_bracket,
                //                             gf,
                //                             gf_reset,
                //                         )
                //                         .expect("grand finals updated")
                //                     }
                //                     _ => dem_partition(&w_bracket, &l_bracket, gf, gf_reset),
                //                 };
                //                 let bracket = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
                //                 let new_matches = new_matches_to_play_for_bracket(
                //                     &old_matches,
                //                     &bracket.matches_to_play(),
                //                 );
                //                 Ok((bracket.matches, new_matches))
                //             }
                //             Err(Error::UnknownMatch(_bad_loser_match)) => {
                //                 let matches = update_grand_finals_or_reset(
                //                     match_id, w_bracket, l_bracket, gf, gf_reset,
                //                 )?;
                //                 let bracket = Step::new(Some(matches), self.seeding.clone(), self.auto)?;
                //                 let new_m = new_matches_to_play_for_bracket(
                //                     &old_matches,
                //                     &bracket.matches_to_play(),
                //                 );
                //                 Ok((bracket.matches, new_m))
                //             }
                //             Err(e) => Err(e),
                //         }
                todo!()
            }
            // Err(e) => Err(e),
            Err(e) => {
                todo!("{e:?}")
            }
        }
    }

    fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .iter()
            .copied()
            .filter(Match::needs_playing)
            .collect()
    }
}

/// when disqualifying a player and updating winner bracket, you can then
/// update loser bracket.
///
/// First you send disqualified player to loser, move him if he was not
/// disqualified, then set him as automatic loser in his current loser bracket
/// match.
fn update_loser_bracket_after_updating_winners_bracket(
    l_bracket: &[Match],
    loser: crate::player::Id,
    is_disqualified_from_winners: bool,
    expected_loser_seed: usize,
) -> Vec<Match> {
    let l_bracket = send_to_losers(l_bracket, loser, expected_loser_seed);
    let l_match = l_bracket
        .iter()
        .find(|m| m.contains(loser))
        .expect("loser match");
    if is_disqualified_from_winners {
        let l_bracket = match crate::bracket::matches::update(&l_bracket, l_match.get_id()) {
            Ok((matches, _)) => matches,
            Err(_) => l_bracket.clone(),
        };
        let l_bracket = match l_bracket
            .iter()
            .find(|m| m.contains(loser) && m.get_winner() == Opponent::Unknown)
        {
            Some(match_to_set_dq) => {
                let match_to_set_dq = (*match_to_set_dq).set_automatic_loser(loser);
                let l_bracket = update_bracket_with(&l_bracket, &match_to_set_dq);
                match crate::bracket::matches::update(&l_bracket, l_match.get_id()) {
                    Ok((matches, _)) => matches,
                    Err(_) => l_bracket,
                }
            }
            // loser finishes in GF
            None => l_bracket,
        };
        l_bracket
    } else {
        let l_bracket = match crate::bracket::matches::update(&l_bracket.clone(), l_match.get_id())
        {
            Ok((l_bracket_matches, _)) => l_bracket_matches,
            Err(_) => l_bracket,
        };
        l_bracket
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
