//! Progression of a double elimination bracket

use crate::bracket::matches::update_bracket_with;
use crate::bracket::progression::new_matches_to_play_for_bracket;
use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::result::Score;
use crate::matches::{Match, MatchID, MatchScore, ReportedResult};
use crate::opponent::Opponent;
use crate::player::PlayerID;
use crate::validation::AutomaticMatchValidationMode;
use thiserror::Error;

/// Error while reporting for a double elimination bracket
#[derive(Debug, Error)]
pub enum DoubleEliminationReportResultError {
    /// Player is disqualified
    ///
    /// Player ID is valid but disqualified player are not allowed to report
    // FIXME add test When a player is DQ'd for the match, they cannot report
    //  that match
    #[error("Player was disqualified {0}")]
    ForbiddenDisqualified(PlayerID),
    /// No match to play for player
    ///
    /// May happen if tournament organiser validated right before player did for
    /// the same match
    // FIXME add test where player has won grand finals but them reporting
    //  results in a message that they won
    #[error("Player has no match to play yet {0}")]
    NoMatchToPlay(PlayerID),
}

impl DoubleEliminationBracket {
    /// Tournament organiser reports `result` for match where `player` is involved.
    ///
    /// Example: player says "I won 2-0" or "I lost 0-2, but it was close though"
    ///
    /// This method is dangerous because not idempotent
    ///
    /// # Panics
    /// FIXME add test When player is unknown
    /// # Errors
    /// FIXME add test When player has played all their matches (won/eliminated)
    /// FIXME use struct `BracketResult` (Unsigned integer x2)
    pub fn tournament_organiser_reports_result_for_single_player_dangerous(
        self,
        player_left: PlayerID,
        bracket_result: MatchScore,
    ) -> Result<(DoubleEliminationBracket, MatchID, Vec<Match>), DoubleEliminationReportResultError>
    {
        todo!()
    }

    /// Report result of player.
    ///
    /// This method is dangerous because it is not idempotent: if you sent the result twice when you
    /// meant to send it once, you can accidentally update two matches.
    ///
    /// # Errors
    /// When bracket state makes report invalid
    /// # Panics
    /// * FIXME add test When player is unknown
    /// * FIXME use struct `BracketResult` (Unsigned integer x2)
    pub fn report_result_dangerous(
        self,
        player_id: PlayerID,
        result: Score,
    ) -> Result<(Vec<Match>, MatchID, Vec<Match>), DoubleEliminationReportResultError> {
        assert!(self.seeding.contains(player_id));
        if crate::bracket::matches::is_disqualified(player_id, &self.matches) {
            return Err(DoubleEliminationReportResultError::ForbiddenDisqualified(
                player_id,
            ));
        }

        let old_matches_to_play = self.matches_to_play();
        let Some(m) = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent(None))
        else {
            return Err(DoubleEliminationReportResultError::NoMatchToPlay(player_id));
        };
        let affected_match_id = m.get_id();
        let bracket =
            self.update_player_reported_match_result(affected_match_id, result, player_id);

        let bracket =
            if bracket.automatic_match_validation_mode == AutomaticMatchValidationMode::Strict {
                bracket
            } else if let Some(match_to_validate) =
                bracket.matches.iter().find(|m| m.id == affected_match_id)
            {
                if match_to_validate.has_all_player_reports() {
                    bracket.validate_match_result(affected_match_id).0
                } else {
                    bracket
                }
            } else {
                panic!()
            };
        // // println!("{:?}", old_matches);
        // // println!("{:?}", p.matches_to_play());
        let new_matches =
            new_matches_to_play_for_bracket(&old_matches_to_play, &bracket.matches_to_play());
        Ok((bracket.matches, affected_match_id, new_matches))
    }

    /// Update `match_id` with reported `result` of `player`
    ///
    /// # Panics
    /// * FIXME add test When `match_id` is unknown
    /// * FIXME add test When `player_id` is unknown
    /// * FIXME use struct `BracketResult` (Unsigned integer x2)
    /// * FIXME add test (0, 0)
    #[must_use]
    pub fn update_player_reported_match_result(
        self,
        match_id: MatchID,
        result: Score,
        player_id: PlayerID,
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
    pub fn validate_match_result(
        self,
        match_id: MatchID,
    ) -> (DoubleEliminationBracket, Vec<Match>) {
        assert_eq!(self.matches.iter().filter(|m| m.id == match_id).count(), 1);
        // NOTE: w_bracket -> winner bracket
        //       l_bracket -> loser bracket
        let (w_bracket, l_bracket, _gf, _gf_reset) =
            self.partition_matches().expect("enough players");
        let match_to_validate_is_in_winner_bracket = w_bracket.iter().any(|m| m.id == match_id);
        let match_to_validate_is_in_loser_bracket = l_bracket.iter().any(|m| m.id == match_id);
        if match_to_validate_is_in_winner_bracket {
            self.validate_from_winner(match_id)
        } else if match_to_validate_is_in_loser_bracket {
            self.validate_from_loser(match_id)
        } else {
            self.validate_from_finals(match_id)
        }
    }

    /// List all matches that can be played out
    pub fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .iter()
            .copied()
            .filter(Match::needs_playing)
            .collect()
    }

    /// `true` if all necessary matches were played
    /// # Panics
    /// When data is corrupted
    #[must_use]
    pub fn is_over(&self) -> bool {
        let (winner_bracket, loser_bracket, gf, gfr) =
            self.partition_matches().expect("enough players");
        let Some(stronger_seed_wins) = gf.stronger_seed_wins() else {
            return false;
        };
        crate::bracket::matches::bracket_is_over(&winner_bracket)
            && crate::bracket::matches::bracket_is_over(&loser_bracket)
            && gf.is_over()
            && (stronger_seed_wins || gfr.is_over())
    }
}

/// when disqualifying a player and updating winner bracket, you can then
/// update loser bracket.
///
/// First you send disqualified player to loser, move him if he was not
/// disqualified, then set him as automatic loser in his current loser bracket
/// match.
pub(crate) fn update_loser_bracket_after_updating_winners_bracket(
    l_bracket: &[Match],
    loser: PlayerID,
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
            .find(|m| m.contains(loser) && m.get_winner() == Opponent(None))
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
        match crate::bracket::matches::update(&l_bracket.clone(), l_match.get_id()) {
            Ok((l_bracket_matches, _)) => l_bracket_matches,
            Err(_) => l_bracket,
        }
    }
}

/// Place loser from winner's bracket into loser bracket using seed of
/// `expected_loser_seed`. Returns updated loser bracket
fn send_to_losers(
    loser_bracket: &[Match],
    loser: PlayerID,
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

impl DoubleEliminationBracket {
    /// Tournament organiser reports result. Returns bracket, affected match ID and new matches
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
    /// # Errors
    /// FIXME add test Reporting result for people that are not playing each other
    pub fn tournament_organiser_reports_result_dangerous(
        self,
        player1: PlayerID,
        result: Score,
        player2: PlayerID,
    ) -> Result<(DoubleEliminationBracket, MatchID, Vec<Match>), DoubleEliminationReportResultError>
    {
        assert!(
            self.seeding.contains(player1),
            "{player1} does not belong in bracket"
        );
        assert!(
            self.seeding.contains(player2),
            "{player2} does not belong in bracket"
        );

        let matches_where_player1_is_playing: Vec<Match> = self
            .clone()
            .matches
            .into_iter()
            .filter(|m| m.contains(player1) && !m.is_over())
            .collect();
        assert!(
            matches_where_player1_is_playing.len() <= 1,
            "player 1 {player1} is involved in only 1 match but they are involved in {matches_where_player1_is_playing:?}",
        );
        let matches_where_player2_is_playing: Vec<Match> = self
            .clone()
            .matches
            .into_iter()
            .filter(|m| m.contains(player2) && !m.is_over())
            .collect();
        assert!(
            matches_where_player2_is_playing.len() <= 1,
            "player 2 {player2} is involved in only 1 match but they are involved in {matches_where_player2_is_playing:?}"
        );

        let bracket = self
            .clone()
            .clear_reported_result(matches_where_player1_is_playing[0].id);

        // report score as p1
        // FIXME should return bracket
        let result_player_1 = ReportedResult(Some(result));
        let (matches, first_affected_match, _new_matches) = bracket
            .report_result_dangerous(player1, result_player_1.0.expect("result"))
            .expect("matches");

        // report same score as p2
        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding.clone(),
            self.automatic_match_validation_mode,
        );

        let (matches, second_affected_match, new_matches) = bracket
            .report_result_dangerous(player2, result_player_1.reverse().0.expect("result"))?;

        assert_eq!(first_affected_match, second_affected_match);

        Ok((
            DoubleEliminationBracket::new(
                matches,
                self.seeding,
                self.automatic_match_validation_mode,
            ),
            first_affected_match,
            new_matches,
        ))
    }
}
