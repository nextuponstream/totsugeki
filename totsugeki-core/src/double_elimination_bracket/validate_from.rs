//! Validation is performed differently according whether match to update is in winners bracket,
//! losers bracket or grand finals/reset

use crate::bracket::matches::Error;
use crate::bracket::progression::new_matches_to_play_for_bracket;
use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::{double_elimination_matches_from_partition, Match};
use crate::opponent::Opponent;
use crate::ID;

impl DoubleEliminationBracket {
    /// Validate bracket with match to update in winners bracket
    pub(crate) fn validate_from_winner(
        self,
        match_id: &ID,
    ) -> (DoubleEliminationBracket, Vec<Match>) {
        let (w_bracket, l_bracket, gf, gf_reset) =
            self.partition_matches().expect("enough players");
        assert!(w_bracket.iter().any(|m| m.get_id() == match_id));
        assert!(!l_bracket.iter().any(|m| m.get_id() == match_id));
        assert_ne!(gf.get_id(), match_id);
        assert_ne!(gf_reset.get_id(), match_id);
        let old_matches_to_play = self.matches_to_play();
        // FIXME make update not a result type
        let (w_bracket, l_bracket_elements) = crate::bracket::matches::update(&w_bracket, match_id)
            .expect("should update winner bracket");
        let l_bracket = match l_bracket_elements {
            Some((loser, expected_loser_seed, is_disqualified_from_winners)) => {
                crate::double_elimination_bracket::progression::update_loser_bracket_after_updating_winners_bracket(
                    &l_bracket,
                    &loser,
                    is_disqualified_from_winners,
                    expected_loser_seed,
                )
            }
            None => l_bracket,
        };

        let gf = match crate::bracket::progression::winner_of_bracket(&w_bracket) {
            Some(winner_of_winner_bracket) => gf.insert_player(&winner_of_winner_bracket, true),
            None => gf,
        };
        // when loser of winners finals is disqualified, grand finals can be updated
        let gf = match crate::bracket::progression::winner_of_bracket(&l_bracket) {
            Some(winner_of_loser_bracket) => {
                let gf = gf.insert_player(&winner_of_loser_bracket, false);

                if w_bracket
                    .iter()
                    .any(|m| m.is_automatic_loser_by_disqualification(&winner_of_loser_bracket))
                {
                    gf.set_automatic_loser(&winner_of_loser_bracket)
                        .update_outcome()
                        .expect("update after automatic disqualification")
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
            crate::bracket::progression::winner_of_bracket(&w_bracket),
            gf.is_over(),
        ) {
            (Opponent(Some(disqualified)), Some(winner_of_winner_bracket), true)
                if *disqualified == winner_of_winner_bracket =>
            {
                Match::new(
                    Some(gf_reset.id),
                    *gf.get_players(),
                    [1, 2],
                    gf.format,
                    Opponent(None),
                    Opponent(None),
                    None,
                )
                .expect("grand final reset")
                .set_automatic_loser(&winner_of_winner_bracket)
                .update_outcome()
                .expect("match update because of disqualified player in it")
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
            new_matches_to_play_for_bracket(&old_matches_to_play, &bracket.matches_to_play());
        (bracket, new_matches)
    }

    /// Validate bracket with match to validate in losers bracket
    pub(crate) fn validate_from_loser(
        self,
        match_id: &ID,
    ) -> (DoubleEliminationBracket, Vec<Match>) {
        let (w_bracket, l_bracket, gf, gf_reset) =
            self.partition_matches().expect("enough players");
        assert!(!w_bracket.iter().any(|m| m.get_id() == match_id));
        assert!(l_bracket.iter().any(|m| m.get_id() == match_id));
        assert_ne!(gf.get_id(), match_id);
        assert_ne!(gf_reset.get_id(), match_id);
        let old_matches_to_play = self.matches_to_play();
        let (l_bracket, _elements) =
            crate::bracket::matches::update(&l_bracket, match_id).expect("update in loser bracket");
        //         send winner of loser bracket to grand finals if
        //         possible
        let gf = match crate::bracket::progression::winner_of_bracket(&l_bracket) {
            Some(winner_of_loser_bracket) => gf.set_player(&winner_of_loser_bracket, false),
            None => gf,
        };
        let matches = match (gf.get_players(), gf.get_automatic_loser()) {
            ([Opponent(Some(_)), Opponent(Some(_))], Opponent(Some(_))) => {
                update_grand_finals_or_reset(gf.get_id(), w_bracket, l_bracket, gf, gf_reset)
                    .expect("grand finals updated")
            }
            _ => double_elimination_matches_from_partition(&w_bracket, &l_bracket, gf, gf_reset),
        };
        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding,
            self.automatic_match_validation_mode,
        );
        let new_matches =
            new_matches_to_play_for_bracket(&old_matches_to_play, &bracket.matches_to_play());
        (bracket, new_matches)
    }

    /// Validate bracket with match to update either be grand finals or reset
    pub(crate) fn validate_from_finals(
        self,
        match_id: &ID,
    ) -> (DoubleEliminationBracket, Vec<Match>) {
        let (w_bracket, l_bracket, gf, gf_reset) =
            self.partition_matches().expect("enough players");
        assert!(!w_bracket.iter().any(|m| m.get_id() == match_id));
        assert!(!l_bracket.iter().any(|m| m.get_id() == match_id));
        assert!(gf.get_id() == match_id || gf_reset.get_id() == match_id);
        let old_matches_to_play = self.matches_to_play();
        let matches = update_grand_finals_or_reset(match_id, w_bracket, l_bracket, gf, gf_reset)
            .expect("grand final or grand final reset should update");
        let bracket = DoubleEliminationBracket::new(
            matches,
            self.seeding,
            self.automatic_match_validation_mode,
        );
        let new_m =
            new_matches_to_play_for_bracket(&old_matches_to_play, &bracket.matches_to_play());
        (bracket, new_m)
    }
}

/// Update grand finals or reset
fn update_grand_finals_or_reset(
    match_id: &ID,
    winner_bracket: Vec<Match>,
    loser_bracket: Vec<Match>,
    gf: Match,
    gf_reset: Match,
) -> Result<Vec<Match>, Error> {
    assert!(!winner_bracket.iter().any(|m| m.get_id() == match_id));
    assert!(!loser_bracket.iter().any(|m| m.get_id() == match_id));
    assert!(gf.get_id() == match_id || gf_reset.get_id() == match_id);
    match match_id {
        id if id == gf.get_id() => {
            let (gf, _, _) = gf.update_outcome()?;
            // when a reset happens in grand finals
            let gf_reset = match (gf.get_winner(), gf.get_players()[1]) {
                (Opponent(Some(gf_winner)), Opponent(Some(player_from_losers)))
                    if *gf_winner == player_from_losers =>
                {
                    // Set players of gf reset
                    let gf_reset = match gf.get_players() {
                        [Opponent(Some(p1)), Opponent(Some(p2))] => {
                            let ggf_reset = gf_reset.insert_player(p1, true);
                            ggf_reset.insert_player(p2, false)
                        }
                        [Opponent(Some(p)), _] => gf_reset.insert_player(p, true),
                        [_, Opponent(Some(p))] => gf_reset.insert_player(p, false),
                        _ => gf_reset,
                    };

                    // if player is disqualified in grand finals, update gf reset
                    match (gf.get_automatic_loser(), gf.get_players()[0]) {
                        (
                            Opponent(Some(grand_finals_loser)),
                            Opponent(Some(winner_of_winner_bracket)),
                        ) if *grand_finals_loser == winner_of_winner_bracket => {
                            gf_reset
                                .set_automatic_loser(grand_finals_loser)
                                .update_outcome()?
                                .0
                        }
                        (_, _) => gf_reset,
                    }
                }
                _ => gf_reset,
            };

            Ok(double_elimination_matches_from_partition(
                &winner_bracket,
                &loser_bracket,
                gf,
                gf_reset,
            ))
        }
        id if id == gf_reset.get_id() => {
            let (gf_reset, _, _) = gf_reset.update_outcome()?;
            Ok([winner_bracket, loser_bracket, vec![gf, gf_reset]].concat())
        }
        _ => panic!("expected GF or GF reset but got other match: {match_id}"),
    }
}
