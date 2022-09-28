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

    /// Takes bracket matches, validate result of match with `match_id`.
    /// Returns updated winner bracket, id of loser (if there is one), the seed
    /// to use when placing them in loser's bracket (if there is one) and the
    /// bracket winner id (if there is one)
    ///
    /// # Errors
    /// thrown when match update is impossible
    #[allow(clippy::type_complexity)]
    // NOTE: you need the info about the person who just lost
    fn update(
        bracket: &[Match],
        match_id: MatchId,
    ) -> Result<(Vec<Match>, Option<(Player, usize)>), Error> {
        if !bracket.iter().any(|m| m.get_id() == match_id) {
            return Err(Error::NoMatchToUpdate(bracket.to_vec(), match_id));
        }
        // declare winner if there is one
        let (updated_match, winner, loser) = match bracket.iter().find(|m| m.get_id() == match_id) {
            Some(m) => m.clone().update_outcome()?,
            None => return Err(Error::UnknownMatch(match_id)),
        };
        let seed_of_expected_winner = updated_match.get_seeds()[0];
        let expected_loser_seed = updated_match.get_seeds()[1];
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
        if last_match.get_id() == match_id {
            if Opponent::Unknown != last_match.get_winner() {
                return Ok((matches, Some((loser, expected_loser_seed))));
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
                return Ok((matches, Some((loser, expected_loser_seed))));
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

        Ok((matches, Some((loser, expected_loser_seed))))
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

    /// Validate match result and return updated bracket with new matches.
    /// Winner moves forward in bracket. If final match is validated, then
    /// bracket will stop accepting match result.
    ///
    /// # Errors
    /// Thrown when given match id is unknown or when reported results differ
    #[allow(clippy::too_many_lines)]
    pub fn validate_match_result(self, match_id: MatchId) -> Result<(Self, Vec<Match>), Error> {
        let old_matches = self.matches_to_play();
        let matches = match self.format {
            SingleElimination => {
                let (matches, _) = Bracket::update(&self.matches, match_id)?;
                matches
            }
            DoubleElimination => {
                let (winner_bracket, loser_bracket, mut gf, gf_reset) =
                    Match::partition_double_elimination_matches(
                        &self.matches,
                        self.participants.len(),
                    )?;
                // find match in winners
                let (winner_bracket, loser_bracket_elements) =
                    match Bracket::update(&winner_bracket, match_id) {
                        Ok(elements) => elements,
                        Err(e) => match e {
                            Error::NoMatchToUpdate(_, _) => {
                                // if not found in winners, look in losers
                                let (loser_bracket, _) =
                                    match Bracket::update(&loser_bracket, match_id) {
                                        Ok(elements) => elements,
                                        Err(e) => match e {
                                            // if not found in losers either
                                            Error::NoMatchToUpdate(_, _) => {
                                                let bracket =
                                                    Bracket::update_grand_finals_or_reset(
                                                        self,
                                                        match_id,
                                                        winner_bracket,
                                                        loser_bracket,
                                                        gf,
                                                        gf_reset,
                                                    )?;

                                                return Ok((
                                                    bracket.clone(),
                                                    new_matches(
                                                        &old_matches,
                                                        &bracket.matches_to_play(),
                                                    ),
                                                ));
                                            }
                                            _ => return Err(e),
                                        },
                                    };
                                let winner_of_loser_bracket = winner_of_bracket(&loser_bracket);
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
                                let new_matches = bracket
                                    .matches_to_play()
                                    .iter()
                                    .filter(|m| {
                                        !old_matches
                                            .iter()
                                            .any(|old_m| old_m.get_id() == m.get_id())
                                    })
                                    .map(std::clone::Clone::clone)
                                    .collect();
                                return Ok((bracket, new_matches));
                            }
                            _ => return Err(e),
                        },
                    };
                let loser_bracket = if let Some((loser, expected_loser_seed)) =
                    loser_bracket_elements
                {
                    let loser_bracket =
                        Bracket::send_to_losers(&loser_bracket, loser.clone(), expected_loser_seed);
                    let loser_match = loser_bracket
                        .iter()
                        .find(|m| m.contains(loser.get_id()))
                        .expect("loser match");
                    if loser_match.get_looser() != Opponent::Unknown
                        && loser_match.get_players()[0] != Opponent::Unknown
                        && loser_match.get_players()[1] != Opponent::Unknown
                        && loser_match.get_winner() == Opponent::Unknown
                    {
                        let (loser_bracket, _) =
                            Bracket::update(&loser_bracket, loser_match.get_id())?;
                        loser_bracket
                    } else {
                        loser_bracket
                    }
                } else {
                    loser_bracket
                };
                let winner_of_winner_bracket = winner_of_bracket(&winner_bracket);
                if let Some(id) = winner_of_winner_bracket {
                    gf = gf.set_player(id, true);
                }
                let winner_of_loser_bracket = winner_of_bracket(&loser_bracket);
                if let Some(id) = winner_of_loser_bracket {
                    gf = gf.set_player(id, false);
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

        Ok((
            bracket.clone(),
            new_matches(&old_matches, &bracket.matches_to_play()),
        ))
    }
}

/// Get new matches using `old_matches` to play and new matches to play
fn new_matches(old_matches: &[Match], new_matches: &[Match]) -> Vec<Match> {
    new_matches
        .iter()
        .filter(|m| !old_matches.iter().any(|old_m| old_m.get_id() == m.get_id()))
        .map(std::clone::Clone::clone)
        .collect()
}

/// Returns winner of bracket
fn winner_of_bracket(bracket: &[Match]) -> Option<Player> {
    match bracket.last() {
        Some(m) => match m.get_winner() {
            Opponent::Player(p) => Some(p),
            Opponent::Unknown => None,
        },
        None => None,
    }
}

#[cfg(test)]
mod tests {

    use crate::{
        bracket::Bracket,
        format::Format,
        opponent::Opponent,
        player::{Id as PlayerId, Player},
        seeding::Method,
    };
    use chrono::prelude::*;

    fn assert_players_play_each_other(
        player_1: usize,
        player_2: usize,
        player_ids: &[PlayerId],
        bracket: &Bracket,
    ) {
        let (next_opponent, match_id_1, _msg) = bracket
            .next_opponent(player_ids[player_1])
            .expect("next opponent");
        if let Opponent::Player(next_opponent) = next_opponent {
            assert_eq!(next_opponent.get_id(), player_ids[player_2]);
        } else {
            panic!("expected player")
        }
        let (next_opponent, match_id_2, _msg) = bracket
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

        let (bracket, _) = bracket.start().expect("start");
        assert_players_play_each_other(2, 3, &player_ids, &bracket);
        let (bracket, _, _) = bracket
            .report_result(player_ids[2], (2, 0))
            .expect("bracket");
        let (_, _, _) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");

        // player 3 reports before TO does
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

        let (bracket, _) = bracket.start().expect("start");
        assert_players_play_each_other(2, 3, &player_ids, &bracket);
        let (bracket, _, _) = bracket
            .report_result(player_ids[3], (0, 2))
            .expect("bracket");
        let (_, _, _) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
    }

    mod single_elimination {
        use crate::{
            bracket::{progression::tests::assert_players_play_each_other, Bracket},
            format::Format,
            player::{Id as PlayerId, Player},
            seeding::Method,
        };
        use chrono::prelude::*;

        #[test]
        fn run_3_man_bracket() {
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

            let (bracket, _) = bracket.start().expect("start");
            assert_eq!(bracket.get_matches().len(), 2);
            assert_eq!(bracket.matches_to_play().len(), 1);
            assert_players_play_each_other(2, 3, &player_ids, &bracket);
            let (bracket, _, new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
                .expect("bracket");
            assert_eq!(new_matches.len(), 1, "grand finals match generated");
            assert_players_play_each_other(1, 2, &player_ids, &bracket);
            assert_eq!(bracket.matches_to_play().len(), 1);
            let (bracket, _, new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
                .expect("bracket");
            assert!(bracket.matches_to_play().is_empty());
            assert!(new_matches.is_empty());
            assert!(bracket.is_over());
        }

        #[test]
        fn run_5_man_bracket() {
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

            let (bracket, _) = bracket.start().expect("start");
            assert_eq!(bracket.get_matches().len(), 4);
            assert_eq!(bracket.matches_to_play().len(), 2);
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
                .expect("bracket");
            assert_eq!(bracket.matches_to_play().len(), 2);
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
                .expect("bracket");
            assert_eq!(bracket.matches_to_play().len(), 1);
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
                .expect("bracket");
            assert_eq!(bracket.matches_to_play().len(), 1);
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
                .expect("bracket");
            assert!(bracket.is_over());
            assert_eq!(bracket.matches_to_play().len(), 0);
        }
    }

    mod double_elimination {
        use crate::{
            bracket::Bracket,
            format::Format,
            matches::Match,
            player::{Id as PlayerId, Player},
            seeding::Method,
        };
        use chrono::prelude::*;

        #[test]
        fn run_3_man_bracket() {
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

            let (bracket, _) = bracket.start().expect("start");
            assert_eq!(bracket.get_matches().len(), 5);
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
                .expect("bracket");
            assert!(bracket.is_over());
        }

        #[test]
        fn partition_matches_for_3_man_bracket() {
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
        fn run_5_man_bracket() {
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

            let (bracket, _) = bracket.start().expect("start");
            assert_eq!(bracket.get_matches().len(), 9);
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[3])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[4])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[5])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[1])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
                .expect("bracket");
            assert!(bracket.is_over());
        }

        #[test]
        fn run_8_man_bracket_no_upsets() {
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

            let (bracket, _) = bracket.start().expect("start");
            assert_eq!(bracket.get_matches().len(), 15);
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[8])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[7])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[5], (2, 0), player_ids[8])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[6], (2, 0), player_ids[7])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[4])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
                .expect("bracket");
            let (bracket, _, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[2])
                .expect("bracket");
            assert!(bracket.is_over());
        }

        #[test]
        fn run_8_man_bracket_with_frequent_upsets() {
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
            let (bracket, _) = bracket.start().expect("start");
            assert_eq!(bracket.get_matches().len(), 15);
            let (bracket, winner_1vs8, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[8])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(winner_1vs8).expect("bracket");
            let (bracket, winner_2vs7, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[7])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(winner_2vs7).expect("bracket");
            let (bracket, winner_3vs6, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(winner_3vs6).expect("bracket");
            let (bracket, winner_4vs5, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(winner_4vs5).expect("bracket");
            let (bracket, loser_4vs8, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[8])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(loser_4vs8).expect("bracket");
            let (bracket, loser_2vs6, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[6])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(loser_2vs6).expect("bracket");
            let (bracket, winner_1vs5, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[5])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(winner_1vs5).expect("bracket");
            let (bracket, winner_3vs7, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[3], (0, 2), player_ids[7])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(winner_3vs7).expect("bracket");
            let (bracket, loser_3vs6, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[6])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(loser_3vs6).expect("bracket");
            let (bracket, loser_4vs5, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[4], (0, 2), player_ids[5])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(loser_4vs5).expect("bracket");
            let (bracket, loser_3vs5, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[3], (2, 0), player_ids[5])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(loser_3vs5).expect("bracket");
            let (bracket, winner_1vs7, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(winner_1vs7).expect("bracket");
            let (bracket, loser_1vs3, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
                .expect("bracket");
            let (bracket, _) = bracket.validate_match_result(loser_1vs3).expect("bracket");
            let (bracket, grand_finals, _new_matches) = bracket
                .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[7])
                .expect("bracket");
            let (bracket, _) = bracket
                .validate_match_result(grand_finals)
                .expect("bracket");
            assert!(bracket.is_over(), "{bracket:?}");
        }
    }
}
