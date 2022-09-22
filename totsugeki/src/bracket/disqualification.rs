//! Disqualification of player in bracket and all side-effects

use crate::{
    bracket::{Bracket, Error},
    matches::{Error as MatchError, Match},
    opponent::Opponent,
    player::Id as PlayerId,
};

impl Bracket {
    /// Disqualify player from bracket, advance opponent in bracket and returns
    /// updated bracket
    ///
    /// # Errors
    /// thrown when referred player does not belong in current bracket, bracket
    /// has not started/is over or participant has already been disqualified
    pub fn disqualify_participant(self, player_id: PlayerId) -> Result<Bracket, Error> {
        if self.is_over() && !self.accept_match_results {
            return Err(Error::AllMatchesPlayed(self.bracket_id));
        }
        if !self.accept_match_results {
            return Err(Error::NotStarted(
                self.bracket_id,
                ". Cannot disqualify at this time.".into(),
            ));
        }

        if let Some(m) = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown)
        {
            let updated_match = m.clone().set_looser(player_id)?;
            let matches = self
                .matches
                .iter()
                .map(|m| {
                    if m.get_id() == updated_match.get_id() {
                        updated_match.clone()
                    } else {
                        m.clone()
                    }
                })
                .collect::<Vec<Match>>();
            let bracket = Self { matches, ..self };
            match bracket
                .clone()
                .validate_match_result(updated_match.get_id())
            {
                Ok(b) => Ok(b),
                Err(bracket_e) => {
                    if let Error::Match(ref e) = bracket_e {
                        match e {
                            // if no winner can be declared because there is a
                            // missing player, then don't throw an error
                            MatchError::MissingOpponent(_) => Ok(bracket),
                            _ => Err(bracket_e),
                        }
                    } else {
                        Err(bracket_e)
                    }
                }
            }
        } else {
            if self.participants.contains(player_id) {
                return Err(Error::PlayerDisqualified(self.bracket_id, player_id));
            }
            Err(Error::UnknownPlayer(
                player_id,
                self.participants.clone(),
                self.bracket_id,
            ))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        bracket::{raw::Raw, Id as BracketId},
        format::Format,
        matches::ReportedResult,
        opponent::Opponent,
        player::Participants,
        seeding::{
            single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
            Method as SeedingMethod,
        },
    };
    use chrono::prelude::*;

    /// Assert x wins against y
    fn assert_outcome(bracket: &Bracket, x: PlayerId, y: PlayerId, x_name: &str, y_name: &str) {
        assert!(
            bracket
                .matches
                .iter()
                .any(|m| if m.contains(x) && m.contains(y) {
                    if let Opponent::Player(p) = m.get_winner() {
                        return p.get_id() == x;
                    }
                    false
                } else {
                    false
                }),
            "No match where {x_name} wins against {y_name}"
        );
    }

    #[test]
    fn cannot_disqualify_player_before_bracket_starts() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        match bracket.disqualify_participant(p1_id) {
            Ok(b) => panic!("Expected error, bracket: {b}"),
            Err(e) => match e {
                Error::NotStarted(id, _) => assert_eq!(id, bracket_id),
                _ => panic!("Expected Started error, got {e}"),
            },
        }
    }

    #[test]
    fn disqualifying_unknown_player_returns_error() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let unknown_player = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let bracket = bracket.start();
        match bracket.disqualify_participant(unknown_player) {
            Ok(b) => panic!("Expected error, bracket: {b}"),
            Err(e) => match e {
                Error::UnknownPlayer(id, _, _) => assert_eq!(id, unknown_player),
                _ => panic!("Expected UnknownPlayer error, got {e}"),
            },
        }
    }

    #[test]
    fn disqualifying_player_that_could_not_make_it() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let bracket = bracket.start();
        assert!(
            !bracket.matches.iter().any(|m| if m.contains(p1_id) {
                if let Opponent::Player(p) = m.get_looser() {
                    return p.get_id() == p1_id;
                }
                false
            } else {
                false
            }),
            "expected player 1 not to be declared looser in any match"
        );
        let bracket = bracket
            .disqualify_participant(p1_id)
            .expect("bracket with player 1 disqualified");
        assert!(
            bracket.matches.iter().any(|m| if m.contains(p1_id) {
                if let Opponent::Player(p) = m.get_looser() {
                    return p.get_id() == p1_id;
                }
                false
            } else {
                false
            }),
            "expected match where player 1 is declared looser"
        );
        assert!(
            bracket
                .matches
                .iter()
                .any(|m| m.contains(p2_id) && m.contains(p3_id)),
            "expected player 2 and 3 playing"
        );
    }

    #[test]
    fn disqualifying_player_sets_looser_of_their_current_match() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let bracket = bracket.start();
        let (bracket, match_id_p2) = bracket
            .report_result(p2_id, ReportedResult((2, 0)))
            .expect("reported result by player 2");
        let (bracket, match_id_p3) = bracket
            .report_result(p3_id, ReportedResult((0, 2)))
            .expect("reported result by player 3");
        assert_eq!(match_id_p2, match_id_p3);
        let bracket = bracket
            .validate_match_result(match_id_p2)
            .expect("validated match for p2 and p3");

        assert!(
            !bracket.matches.iter().any(|m| if m.contains(p2_id) {
                if let Opponent::Player(p) = m.get_looser() {
                    return p.get_id() == p2_id;
                }
                false
            } else {
                false
            }),
            "expected player 2 not to be declared looser in any match"
        );
        let bracket = bracket
            .disqualify_participant(p2_id)
            .expect("p2 is disqualified");
        assert!(
            bracket.matches.iter().any(|m| if m.contains(p2_id) {
                if let Opponent::Player(loser) = m.get_looser() {
                    if loser.get_id() == p2_id {
                        if let Opponent::Player(winner) = m.get_winner() {
                            return winner.get_id() == p1_id;
                        }
                    }
                }
                false
            } else {
                false
            }),
            "expected player 1 winning match where player 2 is disqualified, got {:?}",
            bracket.matches
        );
        assert!(
            bracket
                .matches
                .iter()
                .all(|m| m.get_winner() != Opponent::Unknown),
            "expected all matches were played"
        );
    }

    #[test]
    fn disqualifying_player_sets_their_opponent_as_the_winner_and_they_move_to_their_next_match() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let bracket = bracket.start();
        assert!(
            !bracket.matches.iter().any(|m| if m.contains(p2_id) {
                if let Opponent::Player(p) = m.get_looser() {
                    return p.get_id() == p2_id;
                }
                false
            } else {
                false
            }),
            "expected player 2 not to be declared looser in any match"
        );
        let bracket = bracket
            .disqualify_participant(p2_id)
            .expect("bracket with player 2 disqualified");
        assert!(
            bracket.matches.iter().any(|m| if m.contains(p2_id) {
                if let Opponent::Player(p) = m.get_looser() {
                    return p.get_id() == p2_id;
                }
                false
            } else {
                false
            }),
            "expected match where player 2 is declared looser"
        );
        assert!(
            bracket
                .matches
                .iter()
                .any(|m| m.contains(p1_id) && m.contains(p3_id)),
            "expected player 1 and 3 playing in grand finals"
        );
    }

    #[test]
    fn disqualifying_everyone_is_impossible_because_the_last_player_remaining_wins_grand_finals_automatically(
    ) {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let p4_id = PlayerId::new_v4();
        let p5_id = PlayerId::new_v4();
        let p6_id = PlayerId::new_v4();
        let p7_id = PlayerId::new_v4();
        let p8_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id, p4_id, p5_id, p6_id, p7_id, p8_id];
        let player_names: Vec<String> = vec![
            "p1".into(),
            "p2".into(),
            "p3".into(),
            "p4".into(),
            "p5".into(),
            "p6".into(),
            "p7".into(),
            "p8".into(),
        ];
        let players = Participants::from_raw_id(
            player_ids
                .iter()
                .zip(player_names.iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let bracket_id = BracketId::new_v4();
        let bracket: Bracket = Raw {
            bracket_id,
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        let bracket = bracket.start();
        let bracket = bracket
            .disqualify_participant(p2_id)
            .expect("bracket with player 2 disqualified");
        assert_outcome(&bracket, p7_id, p2_id, "p7", "p2");
        let bracket = bracket
            .disqualify_participant(p3_id)
            .expect("bracket with player 3 disqualified");
        assert_outcome(&bracket, p6_id, p3_id, "p6", "p3");
        let bracket = bracket
            .disqualify_participant(p4_id)
            .expect("bracket with player 4 disqualified");
        assert_outcome(&bracket, p5_id, p4_id, "p5", "p4");
        let bracket = bracket
            .disqualify_participant(p5_id)
            .expect("bracket with player 5 disqualified");
        // player 5 opponent is unknown
        let bracket = bracket
            .disqualify_participant(p6_id)
            .expect("bracket with player 6 disqualified");
        assert_outcome(&bracket, p7_id, p6_id, "p7", "p6");
        let bracket = bracket
            .disqualify_participant(p7_id)
            .expect("bracket with player 7 disqualified");
        // player 7 is in GF
        let bracket = bracket
            .disqualify_participant(p8_id)
            .expect("bracket with player 8 disqualified");
        assert_outcome(&bracket, p1_id, p8_id, "p1", "p8");
        assert_outcome(&bracket, p1_id, p5_id, "p1", "p5");
        assert_outcome(&bracket, p1_id, p7_id, "p1", "p7");

        match bracket.clone().disqualify_participant(p1_id) {
            Ok(_) => panic!("Expected error but none returned: {bracket:?}"),
            Err(e) => match e {
                Error::AllMatchesPlayed(_) => {}
                _ => panic!("Expected AcceptResults error but got {e}"),
            },
        };
    }
}
