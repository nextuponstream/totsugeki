//! Upon match validation, bracket progress by moving winners forward and
//! handling loosers

use super::{Bracket, Error};
use crate::{
    format::Format::{DoubleElimination, SingleElimination},
    matches::{Id as MatchId, Match},
    opponent::Opponent,
    player::Player,
};

impl Bracket {
    /// Update grand finals or reset, assuming winner and loser bracket are
    /// over
    fn update_grand_finals_or_reset(
        bracket: Bracket,
        match_id: MatchId,
        winner_bracket: Vec<Match>,
        loser_bracket: Vec<Match>,
        gf: Match,
        gf_reset: Match,
    ) -> Result<Self, Error> {
        // search in gf and gf reset
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
            let matches = Match::double_elimination_matches_from_partition(
                &winner_bracket,
                &loser_bracket,
                gf,
                gf_reset,
            );
            return Ok(Self {
                matches,
                accept_match_results,
                ..bracket
            });
        } else if gf_reset.get_id() == match_id {
            let (gf_reset, _, _) = gf_reset.update_outcome()?;
            let mut matches = winner_bracket;
            matches.append(&mut loser_bracket);
            matches.push(gf);
            matches.push(gf_reset);
            return Ok(Self {
                matches,
                accept_match_results: false,
                ..bracket
            });
        }
        Err(Error::NoMatchToUpdate(bracket.matches, match_id))
    }

    /// Takes bracket matches, validate result of match with `match_id` and
    /// returns updated winner bracket, id of loser, the seed to use when
    /// placing them in loser's bracket (if needed) and the bracket winner id
    ///
    /// # Errors
    /// thrown when match update is impossible
    fn update(
        bracket: &[Match],
        match_id: MatchId,
    ) -> Result<(Vec<Match>, Player, usize, Option<Player>), Error> {
        if !bracket.iter().any(|m| m.get_id() == match_id) {
            return Err(Error::NoMatchToUpdate(bracket.to_vec(), match_id));
        }
        // declare winner if there is one
        let (updated_match, winner, loser) = match bracket.iter().find(|m| m.get_id() == match_id) {
            Some(m) => m.clone().update_outcome()?,
            None => return Err(Error::UnknownMatch(match_id)),
        };
        let seed_of_expected_winner = updated_match.get_seeds()[0];
        let matches: Vec<_> = bracket
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match.clone()
                } else {
                    m.clone()
                }
            })
            .collect();

        let last_match = matches.last().expect("last match");
        let expected_loser_seed = updated_match.get_seeds()[1];
        if last_match.get_id() == match_id {
            if let Opponent::Player(p) = last_match.get_winner() {
                return Ok((matches, loser, expected_loser_seed, Some(p)));
            }
            panic!("No winner of bracket");
        }

        // winner moves forward in bracket
        let index = matches
            .iter()
            .position(|m| m.get_id() == updated_match.get_id())
            .expect("reference to updated match");
        let mut iter = matches.iter().skip(index + 1);
        let m = iter
            .find(|m| m.get_seeds().contains(&seed_of_expected_winner))
            .expect("match where winner plays next");
        let updated_match = m
            .clone()
            .set_player(winner.clone(), m.get_seeds()[0] == seed_of_expected_winner);
        let mut matches: Vec<Match> = matches
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match.clone()
                } else {
                    m.clone()
                }
            })
            .collect();

        // looser drops to loser bracket in double elimination format

        // Set winner to all matches were a player is disqualified
        // while loop is needed because there can be a scenario where a player
        // is moved up several times because each next match contains a
        // disqualified player
        while matches
            .iter()
            .any(Match::needs_update_because_of_disqualified_participant)
        {
            let match_id = matches
                .iter()
                .find(|m| m.needs_update_because_of_disqualified_participant())
                .expect("match with disqualified player")
                .get_id();
            let (updated_match, _winner_id, _) =
                match matches.iter().find(|m| m.get_id() == match_id) {
                    Some(m) => m.clone().update_outcome()?,
                    None => return Err(Error::UnknownMatch(match_id)),
                };
            matches = matches
                .iter()
                .map(|m| {
                    if m.get_id() == updated_match.get_id() {
                        updated_match.clone()
                    } else {
                        m.clone()
                    }
                })
                .collect();

            let last_match = matches.last().expect("last match");
            if last_match.get_id() == match_id {
                return Ok((matches, loser, expected_loser_seed, None));
            }

            // winner moves forward in bracket
            let index = matches
                .iter()
                .position(|m| m.get_id() == updated_match.get_id())
                .expect("reference to updated match");
            let mut iter = matches.iter().skip(index + 1);
            let seed_of_expected_winner = updated_match.get_seeds()[0];
            let m = iter
                .find(|m| m.get_seeds().contains(&seed_of_expected_winner))
                .expect("match where winner plays next");
            let updated_match = m
                .clone()
                .set_player(winner.clone(), m.get_seeds()[0] == seed_of_expected_winner);
            matches = matches
                .iter()
                .map(|m| {
                    if m.get_id() == updated_match.get_id() {
                        updated_match.clone()
                    } else {
                        m.clone()
                    }
                })
                .collect();
        }

        Ok((matches, loser, expected_loser_seed, None))
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
            .expect("loser first match in losers");
        let is_player_1 = expected_loser_seed == loser_match.get_seeds()[0];
        let loser_match = loser_match.clone().set_player(loser, is_player_1);
        loser_bracket
            .iter()
            .map(|m| {
                if m.get_id() == loser_match.get_id() {
                    loser_match.clone()
                } else {
                    m.clone()
                }
            })
            .collect()
    }

    /// Validate match result and return updated bracket. Winner moves forward
    /// in bracket. If final match is validated, then bracket will stop
    /// accepting match result.
    ///
    /// # Errors
    /// Thrown when given match id is unknown or when reported results differ
    pub fn validate_match_result(self, match_id: MatchId) -> Result<Self, Error> {
        let matches = match self.format {
            SingleElimination => {
                let (matches, _, _, _) = Bracket::update(&self.matches, match_id)?;
                matches
            }
            DoubleElimination => {
                let (winner_bracket, loser_bracket, mut gf, gf_reset) =
                    Match::partition_double_elimination_matches(
                        &self.matches,
                        self.participants.len(),
                    )?;
                // find match in winners
                let (winner_bracket, loser, expected_loser_seed, winner_of_winner_bracket) =
                    match Bracket::update(&winner_bracket, match_id) {
                        Ok(elements) => elements,
                        Err(e) => match e {
                            Error::NoMatchToUpdate(_, _) => {
                                // if not found in winners, look in losers
                                let (loser_bracket, _, _, winner_of_loser_bracket) =
                                    match Bracket::update(&loser_bracket, match_id) {
                                        Ok(elements) => elements,
                                        Err(e) => match e {
                                            // if not found in losers either
                                            Error::NoMatchToUpdate(_, _) => {
                                                return Bracket::update_grand_finals_or_reset(
                                                    self,
                                                    match_id,
                                                    winner_bracket,
                                                    loser_bracket,
                                                    gf,
                                                    gf_reset,
                                                )
                                            }
                                            _ => return Err(e),
                                        },
                                    };
                                // try forming grand finals
                                gf = match winner_of_loser_bracket {
                                    Some(w) => gf.set_opponent(w, false),
                                    None => gf,
                                };
                                let matches = Match::double_elimination_matches_from_partition(
                                    &winner_bracket,
                                    &loser_bracket,
                                    gf,
                                    gf_reset,
                                );
                                let bracket = Self { matches, ..self };

                                return Ok(bracket);
                            }
                            _ => return Err(e),
                        },
                    };
                let loser_bracket =
                    Bracket::send_to_losers(&loser_bracket, loser, expected_loser_seed);
                if let Some(id) = winner_of_winner_bracket {
                    gf = gf.set_player(id, true);
                }
                Match::double_elimination_matches_from_partition(
                    &winner_bracket,
                    &loser_bracket,
                    gf,
                    gf_reset,
                )
            }
        };

        let bracket = Self { matches, ..self };
        let bracket = Self {
            accept_match_results: !bracket.is_over(),
            ..bracket
        };

        Ok(bracket)
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        bracket::Bracket,
        format::Format,
        matches::Match,
        player::{Id as PlayerId, Player},
        seeding::Method,
    };
    use chrono::prelude::*;

    #[test]
    fn progress_single_elimination_3_man_bracket() {
        let mut bracket = Bracket::new(
            "",
            Format::SingleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.add_new_player(player).expect("bracket");
        }

        bracket = bracket.start();
        assert_eq!(bracket.get_matches().len(), 2);
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, m_2vs3) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(m_2vs3).expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, m_1vs2) = bracket
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
            .expect("bracket");
        let bracket = bracket.validate_match_result(m_1vs2).expect("bracket");
        assert!(bracket.matches_to_play().is_empty());
        assert!(bracket.is_over());
    }

    #[test]
    fn progress_single_elimination_5_man_bracket() {
        let mut bracket = Bracket::new(
            "",
            Format::SingleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.add_new_player(player).expect("bracket");
        }

        bracket = bracket.start();
        assert_eq!(bracket.get_matches().len(), 4);
        assert_eq!(bracket.matches_to_play().len(), 2);
        let (bracket, m_4vs5) = bracket
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(m_4vs5).expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 2);
        let (bracket, m_2vs3) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(m_2vs3).expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, m_1vs4) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
            .expect("bracket");
        let bracket = bracket.validate_match_result(m_1vs4).expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, m_1vs3) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(m_1vs3).expect("bracket");
        assert!(bracket.is_over());
        assert_eq!(bracket.matches_to_play().len(), 0);
    }

    #[test]
    fn partition_double_elimination_matches_3_man_bracket() {
        let mut bracket = Bracket::new(
            "",
            Format::DoubleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.add_new_player(player).expect("bracket");
        }

        let (winner_bracket, loser_bracket, _gf, _gfr) =
            Match::partition_double_elimination_matches(
                &bracket.get_matches(),
                bracket.get_participants().len(),
            )
            .expect("partition");
        assert_eq!(winner_bracket.len(), 2);
        assert_eq!(loser_bracket.len(), 1);
        assert_eq!(loser_bracket[0].get_seeds(), [2, 3]);
    }

    #[test]
    fn progress_double_elimination_3_man_bracket() {
        let mut bracket = Bracket::new(
            "",
            Format::DoubleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.add_new_player(player).expect("bracket");
        }

        bracket = bracket.start();
        assert_eq!(bracket.get_matches().len(), 5);
        let (bracket, winner_2vs3) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_2vs3).expect("bracket");
        let (bracket, winner_1vs2) = bracket
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs2).expect("bracket");
        let (bracket, looser_1vs3) = bracket
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(looser_1vs3).expect("bracket");
        println!("{:?}", bracket.matches);
        let (bracket, grand_finals) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let bracket = bracket
            .validate_match_result(grand_finals)
            .expect("bracket");
        let (bracket, grand_finals_reset) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let bracket = bracket
            .validate_match_result(grand_finals_reset)
            .expect("bracket");
        assert!(bracket.is_over());
    }

    #[test]
    fn progress_double_elimination_5_man_bracket() {
        let mut bracket = Bracket::new(
            "",
            Format::DoubleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.add_new_player(player).expect("bracket");
        }

        bracket = bracket.start();
        assert_eq!(bracket.get_matches().len(), 9);
        let (bracket, winner_2vs3) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_2vs3).expect("bracket");
        let (bracket, winner_4vs5) = bracket
            .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_4vs5).expect("bracket");
        let (bracket, winner_1vs5) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs5).expect("bracket");
        let (bracket, winner_1vs3) = bracket
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs3).expect("bracket");
        let (bracket, looser_4vs5) = bracket
            .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[4])
            .expect("bracket");
        let bracket = bracket.validate_match_result(looser_4vs5).expect("bracket");
        let (bracket, looser_2vs5) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(looser_2vs5).expect("bracket");
        let (bracket, looser_1vs2) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[1])
            .expect("bracket");
        let bracket = bracket.validate_match_result(looser_1vs2).expect("bracket");
        let (bracket, grand_finals) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        let bracket = bracket
            .validate_match_result(grand_finals)
            .expect("bracket");
        assert!(bracket.is_over());
    }

    #[test]
    fn progress_double_elimination_8_man_bracket_no_upsets() {
        let mut bracket = Bracket::new(
            "",
            Format::DoubleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            true,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.add_new_player(player).expect("bracket");
        }

        bracket = bracket.start();
        assert_eq!(bracket.get_matches().len(), 15);
        let (bracket, winner_1vs8) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[8])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs8).expect("bracket");
        let (bracket, winner_2vs7) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[7])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_2vs7).expect("bracket");
        let (bracket, winner_3vs6) = bracket
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_3vs6).expect("bracket");
        let (bracket, winner_4vs5) = bracket
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_4vs5).expect("bracket");
        let (bracket, loser_5vs8) = bracket
            .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[8])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_5vs8).expect("bracket");
        let (bracket, loser_6vs7) = bracket
            .tournament_organiser_reports_result(player_ids[6], (2, 0), player_ids[7])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_6vs7).expect("bracket");
        let (bracket, winner_1vs4) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs4).expect("bracket");
        let (bracket, winner_2vs3) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_2vs3).expect("bracket");
        let (bracket, loser_3vs6) = bracket
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_3vs6).expect("bracket");
        let (bracket, loser_4vs5) = bracket
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_4vs5).expect("bracket");
        let (bracket, loser_3vs4) = bracket
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[4])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_3vs4).expect("bracket");
        let (bracket, winner_1vs2) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs2).expect("bracket");
        let (bracket, loser_2vs3) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_2vs3).expect("bracket");
        let (bracket, grand_finals) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
            .expect("bracket");
        let bracket = bracket
            .validate_match_result(grand_finals)
            .expect("bracket");
        assert!(bracket.is_over());
    }

    #[test]
    fn progress_double_elimination_8_man_bracket_with_frequent_upsets() {
        // every 2 matches, there is an upset
        let mut bracket = Bracket::new(
            "",
            Format::DoubleElimination,
            Method::Strict,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        );
        let mut player_ids = vec![PlayerId::new_v4()]; // padding for readability
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bracket = bracket.add_new_player(player).expect("bracket");
        }
        bracket = bracket.start();
        assert_eq!(bracket.get_matches().len(), 15);
        let (bracket, winner_1vs8) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[8])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs8).expect("bracket");
        let (bracket, winner_2vs7) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[7])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_2vs7).expect("bracket");
        let (bracket, winner_3vs6) = bracket
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_3vs6).expect("bracket");
        let (bracket, winner_4vs5) = bracket
            .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_4vs5).expect("bracket");
        let (bracket, loser_4vs8) = bracket
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[8])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_4vs8).expect("bracket");
        let (bracket, loser_2vs6) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[6])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_2vs6).expect("bracket");
        let (bracket, winner_1vs5) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs5).expect("bracket");
        let (bracket, winner_3vs7) = bracket
            .tournament_organiser_reports_result(player_ids[3], (0, 2), player_ids[7])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_3vs7).expect("bracket");
        let (bracket, loser_3vs6) = bracket
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_3vs6).expect("bracket");
        let (bracket, loser_4vs5) = bracket
            .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_4vs5).expect("bracket");
        let (bracket, loser_3vs5) = bracket
            .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[5])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_3vs5).expect("bracket");
        let (bracket, winner_1vs7) = bracket
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
            .expect("bracket");
        let bracket = bracket.validate_match_result(winner_1vs7).expect("bracket");
        let (bracket, loser_1vs3) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
            .expect("bracket");
        let bracket = bracket.validate_match_result(loser_1vs3).expect("bracket");
        let (bracket, grand_finals) = bracket
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
            .expect("bracket");
        let bracket = bracket
            .validate_match_result(grand_finals)
            .expect("bracket");
        assert!(bracket.is_over(), "{bracket:?}");
    }
}
