//! Query state of bracket

mod double_elimination;
mod single_elimination;

use crate::{
    bracket::{Bracket, Error},
    format::Format::{DoubleElimination, SingleElimination},
    matches::{Id as MatchId, Match},
    opponent::Opponent,
    player::{Id as PlayerId, Player},
};

impl Bracket {
    /// Returns true if player has been disqualified from bracket
    #[must_use]
    pub(super) fn is_disqualified(&self, player_id: PlayerId) -> bool {
        self.matches
            .iter()
            .any(|m| m.is_automatic_looser_by_disqualification(player_id))
    }

    /// Returns true if bracket is over
    fn bracket_is_over(bracket_matches: &[Match]) -> bool {
        !bracket_matches.iter().any(|m| !m.is_over())
    }

    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    pub(super) fn is_over(&self) -> bool {
        match self.format {
            SingleElimination => Self::bracket_is_over(&self.matches),
            DoubleElimination => {
                let (winner_bracket, looser_bracket, gf, gfr) =
                    Match::partition_double_elimination_matches(
                        &self.matches,
                        self.participants.len(),
                    )
                    .expect("partition");
                Self::bracket_is_over(&winner_bracket)
                    && Self::bracket_is_over(&looser_bracket)
                    && gf.is_over()
                    && (gf.stronger_seed_wins() || gfr.is_over())
            }
        }
    }

    /// Return next opponent for `player_id`, relevant match and player name
    ///
    /// # Errors
    /// Thrown when matches have yet to be generated or player has won/been
    /// eliminated
    // FIXME don't return player name
    pub fn next_opponent(&self, player_id: PlayerId) -> Result<(Opponent, MatchId, String), Error> {
        if !self.participants.contains(player_id) {
            return Err(Error::PlayerIsNotParticipant(player_id, self.bracket_id));
        }
        if self.matches.is_empty() {
            return Err(Error::NoGeneratedMatches(self.bracket_id));
        }

        if self.is_disqualified(player_id) {
            return Err(Error::DisqualifiedPlayerHasNoNextOpponent(
                self.bracket_id,
                player_id,
            ));
        }

        let next_match = self
            .matches
            .iter()
            .find(|m| m.contains(player_id) && m.get_winner() == Opponent::Unknown);
        let relevant_match = if let Some(m) = next_match {
            m
        } else {
            // FIXME correct winner of double elimination
            // if self.format() == Format::DoubleElimination {
            //     if bracket.
            // }
            let last_match = self.matches.iter().last().expect("last match");
            if let Opponent::Player(p) = last_match.get_winner() {
                if p.get_id() == player_id {
                    return Err(Error::NoNextMatch(player_id, self.bracket_id));
                }
            }
            return Err(Error::EliminatedFromBracket(player_id, self.bracket_id));
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
        let player_name = match opponent.clone() {
            Opponent::Player(opponent) => self
                .participants
                .clone()
                .get_players_list()
                .iter()
                .find(|p| p.id == opponent.get_id())
                .map_or_else(|| Opponent::Unknown.to_string(), Player::get_name),
            Opponent::Unknown => Opponent::Unknown.to_string(),
        };

        Ok((opponent, relevant_match.get_id(), player_name))
    }
}

#[cfg(test)]
use crate::{format::Format, seeding::Method};
#[cfg(test)]
use chrono::prelude::*;
#[cfg(test)]
fn create_bracket_with_n_players_and_start(
    n: usize,
    format: Format,
    seeding_method: Method,
    automatic_match_validation: bool,
) -> (Bracket, Vec<Player>) {
    let mut players = vec![Player::new("don't use".into())];
    let mut bracket = Bracket::new(
        "",
        format,
        seeding_method,
        Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
        automatic_match_validation,
    );
    for i in 1..=n {
        let player = Player::new(format!("p{i}"));
        players.push(player.clone());
        bracket = bracket.add_new_player(player).expect("bracket");
    }
    bracket = bracket.start();

    (bracket, players)
}

#[cfg(test)]
fn assert_elimination(bracket: &Bracket, players: &[Player], player_who_won: usize) {
    let iter = players.iter().enumerate();
    let iter = iter.skip(1);

    for (i, p) in iter {
        let e = bracket
            .next_opponent(p.get_id())
            .expect_err("EliminatedFromBracket/NoNextMatch");
        if i == player_who_won {
            if let Error::NoNextMatch(player_id, _) = e {
                assert_eq!(player_id, p.get_id());
            } else {
                panic!("expected NoNextMatch error but got {e}");
            }
        } else if let Error::EliminatedFromBracket(player_id, _) = e {
            assert_eq!(player_id, p.get_id());
        } else {
            panic!("expected EliminatedFromBracket error but got {e}");
        }
    }
}

#[cfg(test)]
fn assert_next_matches(
    bracket: &Bracket,
    players_with_unknown_opponent: &[usize],
    expected_matches: &[(usize, usize)],
    players: &[Player],
) {
    for p in players_with_unknown_opponent {
        let player = players[*p].clone();
        let (next_opponent, _, _) = bracket
            .next_opponent(player.get_id())
            .expect("next opponent");
        assert_eq!(
            next_opponent,
            Opponent::Unknown,
            "expected unknown opponent for {p} but got {next_opponent}"
        );
    }

    for (o1, o2) in expected_matches {
        let opponent1 = players[*o1].clone();
        let opponent2 = players[*o2].clone();

        let (next_opponent, _, _) = bracket
            .next_opponent(opponent1.get_id())
            .expect("next opponent");
        if let Opponent::Player(p) = next_opponent {
            assert_eq!(
                p.get_id(),
                opponent2.get_id(),
                "expected {opponent2} for {opponent1} but got {p}"
            );
        } else {
            panic!("expected player for next opponent");
        }
        let (next_opponent, _, _) = bracket
            .next_opponent(opponent2.get_id())
            .expect("next opponent");
        if let Opponent::Player(p) = next_opponent {
            assert_eq!(
                p.get_id(),
                opponent1.get_id(),
                "expected {opponent1} for {opponent2} but got {p}"
            );
        } else {
            panic!("expected player for next opponent");
        }
    }
}
