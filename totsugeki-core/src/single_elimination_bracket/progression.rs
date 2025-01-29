//! Progression of a single elimination bracket

use crate::bracket::matches::{bracket_is_over, Error};
use crate::bracket::progression::new_matches_to_play_for_bracket;
use crate::matches::result::Score;
use crate::matches::Error as MatchError;
use crate::matches::{Match, ReportedResult};
use crate::single_elimination_bracket::{
    SingleEliminationBracket, SingleEliminationReportResultError,
};
use crate::ID;
// FIXME add all test for reports from double elimination here too

// TODO force implementation of score report where you are required to tell all players involved
//  rather then inferring (p1, p2). This way, does additional checks are done (is p2
//  disqualified?). Currently, it only requires p1, which is fine in itself. There might be a
//  case to require all players involved that I don't foresee, like a performance improvement

// TODO for consistency, make Progression trait common to single elim and double elim but MAKE IT
//  CLEAR that the abstraction is only for library DX and it should be taken out once both
//  implementations diverge

impl SingleEliminationBracket {
    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    pub fn is_over(&self) -> bool {
        bracket_is_over(&self.matches)
    }

    /// Returns true if player is disqualified
    #[must_use]
    pub fn is_disqualified(&self, player_id: &ID) -> bool {
        self.matches
            .iter()
            .any(|m| m.is_automatic_loser_by_disqualification(player_id))
    }

    // /// Report result of match. Returns updated matches, affected match and new
    // /// matches to play
    // /// # Errors
    // /// thrown when player does not belong in bracket
    // /// # Panics
    // /// When `player_id` is unknown
    // fn report_result(
    //     self,
    //     player_id: PlayerID,
    //     result: Score,
    // ) -> Result<(Vec<Match>, MatchID, Vec<Match>), SingleEliminationReportResultError> {
    //     assert!(
    //         self.seeding.contains(player_id),
    //         "Unknown player {player_id}"
    //     );
    //     if self.is_over() {
    //         return Err(SingleEliminationReportResultError::TournamentIsOver);
    //     }
    //     if self.is_disqualified(player_id) {
    //         return Err(SingleEliminationReportResultError::ForbiddenDisqualified(
    //             player_id,
    //         ));
    //     }
    //     let match_to_update = self
    //         .matches
    //         .iter()
    //         .find(|m| m.contains(player_id) && m.get_winner() == Opponent(None));
    //     todo!()
    // match match_to_update {
    //     Some(m) => {
    //         let old_matches = self.matches_to_play();
    //         let affected_match_id = m.get_id();
    //         let matches =
    //             self.update_player_reported_match_result(affected_match_id, result, player_id)?;
    //         // let p = crate::bracket::matches::single_elimination_format::Step::new(
    //         //     Some(matches),
    //         //     &self.seeding,
    //         //     self.automatic_progression,
    //         // )?;
    //
    //         let matches = if self.automatic_match_progression {
    //             // match p.clone().validate_match_result(affected_match_id) {
    //             //     Ok((b, _)) => b,
    //             //                 Err(e) => match e {
    //             //                     Error::MatchUpdate(
    //             //                         crate::matches::Error::PlayersReportedDifferentMatchOutcome(_, _),
    //             //                     ) => p.matches,
    //             //                     _ => return Err(e),
    //             //                 },
    //             //             }
    //             todo!()
    //         } else {
    //             // p.matches
    //             self.matches.clone()
    //         };
    //         //
    //         //         let p = crate::bracket::matches::single_elimination_format::Step::new(
    //         //             Some(matches),
    //         //             &self.seeding,
    //         //             self.automatic_progression,
    //         //         )?;
    //         //
    //         //         let new_matches = p
    //         //             .matches_to_play()
    //         //             .iter()
    //         //             .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
    //         //             .map(std::clone::Clone::clone)
    //         //             .collect();
    //         //         Ok((p.matches, affected_match_id, new_matches))
    //         todo!()
    //     }
    //     None => Err(SingleEliminationReportResultError::NoMatchToPlay(player_id)),
    // }
    // }

    /// Update `match_id` with reported `result` of `player`
    ///
    /// # Panics
    /// When `match_id` or `player_id` is unknown
    /// # Errors
    /// Input is invalid for this bracket state
    pub fn update_player_reported_match_result(
        self,
        match_id: &ID,
        result: Score,
        player_id: &ID,
    ) -> Result<Vec<Match>, SingleEliminationReportResultError> {
        let Some(m) = self.matches.iter().find(|m| m.get_id() == match_id) else {
            panic!("unknown match {match_id}")
        };
        assert!(m.contains(player_id), "{player_id} is not in match");

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

    /// Returns updated bracket and new matches to play. Uses `match_id` as the
    /// first match to start updating before looking deeper into the bracket
    #[must_use]
    pub fn validate_match_result(self, match_id: &ID) -> (SingleEliminationBracket, Vec<Match>) {
        let old_matches_to_play = self.matches_to_play();
        // FIXME remove unreachable
        // FIXME should return an error because it's used by a library => there must be some
        //  valid input that becomes invalid depending on the state the bracket is in
        let (matches, _) = match crate::bracket::matches::update(&self.matches, match_id) {
            Ok(t) => t,
            Err(Error::MatchUpdate(
                MatchError::MissingOpponent(_) | MatchError::MissingReport(_, _),
            )) => return (self, vec![]),
            Err(e) => unreachable!("{e:?}"),
        };

        let bracket = SingleEliminationBracket::new(
            self.seeding,
            matches,
            self.automatic_match_validation_mode,
        );
        let new_matches =
            new_matches_to_play_for_bracket(&old_matches_to_play, &bracket.matches_to_play());
        (bracket, new_matches)
    }

    /// List all matches that can be played out
    pub fn matches_to_play(&self) -> Vec<Match> {
        self.matches
            .iter()
            .copied()
            .filter(Match::needs_playing)
            .collect()
    }

    // // FIXME redundant with next_opponent_in_bracket
    // fn next_opponent(&self, player_id: PlayerID) -> Option<(Opponent, MatchID)> {
    //     assert!(self.seeding.contains(player_id), "unknown player");
    //
    //     if self.matches.is_empty() {
    //         unreachable!()
    //     }
    //
    //     if is_disqualified(player_id, &self.matches) {
    //         return None;
    //     }
    //
    //     let next_match = self
    //         .matches
    //         .iter()
    //         .find(|m| m.contains(player_id) && m.get_winner() == Opponent(None));
    //     let relevant_match = next_match?;
    //
    //     let opponent = match &relevant_match.get_players() {
    //         [Opponent(Some(p1)), Opponent(Some(p2))] if *p1 == player_id => Opponent(Some(*p2)),
    //         [Opponent(Some(p1)), Opponent(Some(p2))] if *p2 == player_id => Opponent(Some(*p1)),
    //         _ => Opponent(None),
    //     };
    //     Some((opponent, relevant_match.get_id()))
    // }

    // FIXME return self and consume...
    /// Tournament organiser reports result. Returns updated bracket, affected
    /// `match_id` and new matches to play (that weren't playable before)
    ///
    /// NOTE: both players are needed, so it is less ambiguous when reading code:
    /// * p1 2-0 is more ambiguous to read than
    /// * p1 2-0 p2
    ///
    /// Technically, it's unnecessary.
    ///
    /// Returns updated bracked, match ID where score was applied and newly
    /// updated matches
    ///
    /// # Errors
    /// thrown when player does not belong in bracket
    /// # Panics
    /// When either `player1` or `player2` is unknown
    pub fn tournament_organiser_reports_result(
        self,
        player1: &ID,
        result: Score,
        player2: &ID,
    ) -> Result<(SingleEliminationBracket, ID, Vec<Match>), SingleEliminationReportResultError>
    {
        let result_player_1 = ReportedResult(Some(result));
        let bracket = self.clone().clear_reported_result(player1);
        let bracket = bracket.clear_reported_result(player2);
        let (bracket, first_affected_match, _new_matches) =
            bracket.report_result(player1, result)?;
        let (bracket, second_affected_match, new_matches_2) =
            bracket.report_result(player2, result_player_1.reverse().0.expect("result"))?;
        assert_eq!(first_affected_match, second_affected_match);
        Ok((bracket, first_affected_match, new_matches_2))
    }
}

#[cfg(test)]
mod tests {
    use crate::bracket::seeding::Seeding;
    use crate::matches::result::{MatchFormat, Score};
    use crate::next_opponent::NextOpponentInBracket;
    use crate::opponent::Opponent;
    use crate::player::Player;
    use crate::seeding::single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored;
    use crate::single_elimination_bracket::SingleEliminationBracket;
    use crate::validation::AutomaticMatchValidationMode;

    fn assert_players_play_each_other(
        player_1: usize,
        player_2: usize,
        player_ids: &[Player],
        s: &SingleEliminationBracket,
    ) {
        let (next_opponent, match_id_1) = s
            .next_opponent_in_bracket(player_ids[player_1].get_id())
            .unwrap();
        let Opponent(Some(next_opponent)) = next_opponent else {
            panic!("expected player");
        };
        assert_eq!(next_opponent, *player_ids[player_2].get_id());

        let (next_opponent, match_id_2) = s
            .next_opponent_in_bracket(player_ids[player_2].get_id())
            .unwrap();
        let Opponent(Some(next_opponent)) = next_opponent else {
            panic!("expected player")
        };
        assert_eq!(next_opponent, *player_ids[player_1].get_id());

        assert_eq!(
            match_id_1, match_id_2,
            "expected player to be playing the same match"
        );
    }

    #[test]
    fn run_3_man() {
        let mut p = vec![Player::new("don't use".into())]; // padding for readability
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id().to_owned());
        }
        let seeding = Seeding::new(seeding).unwrap();
        let auto = AutomaticMatchValidationMode::Flexible;
        let matches =
            get_balanced_round_matches_top_seed_favored(&seeding, MatchFormat::ft2(), None);
        let bracket = SingleEliminationBracket::new(seeding, matches, auto);

        assert_eq!(bracket.matches.len(), 2);
        assert_eq!(bracket.matches_to_play().len(), 1);
        assert_players_play_each_other(2, 3, &p, &bracket);
        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[2].get_id(), Score(2, 0), p[3].get_id())
            .expect("bracket");
        assert_eq!(new_matches.len(), 1, "grand finals match generated");
        assert_players_play_each_other(1, 2, &p, &bracket);
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), Score(0, 2), p[2].get_id())
            .expect("bracket");
        assert!(bracket.matches_to_play().is_empty());
        assert!(new_matches.is_empty());
        assert!(bracket.is_over());
    }
}
