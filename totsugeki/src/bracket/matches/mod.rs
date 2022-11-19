//! Manage matches from bracket

use crate::{
    format::Format,
    matches::{Error as MatchError, Id as MatchId, Match},
    opponent::Opponent,
    player::{Id as PlayerId, Participants, Player},
    seeding::Error as SeedingError,
};
use thiserror::Error;

pub mod double_elimination_format;
pub mod single_elimination_format;

/// Error while managing matches of bracket
#[derive(Error, Debug)]
pub enum Error {
    /// Seeding error
    #[error("{0}")]
    Seeding(#[from] SeedingError),
    /// Cannot request information from unknown player
    #[error("Player {0} is unknown in this bracket")]
    PlayerIsNotParticipant(PlayerId),
    /// There is no generated matches at this time
    #[error("No matches were generated yet")]
    NoGeneratedMatches,
    /// Player has been disqualified
    #[error("{0} has been disqualified")]
    Disqualified(Player),
    /// Player has won the tournament and has no match left to play
    #[error("{0} won the tournament and has no matches left to play")]
    NoNextMatch(Player),
    /// Player is eliminated from tournament and has no matches left to play
    #[error("{0} has been eliminated from the tournament and has no matches left to play")]
    Eliminated(Player),
    /// Tournament is over
    #[error("Tournament is over")]
    TournamentIsOver,
    /// Cannot update match
    #[error("{0}")]
    MatchUpdate(#[from] MatchError),
    /// Player is disqualified
    #[error("{0} is disqualified")]
    PlayerDisqualified(Player),
    /// Player is unknown in this bracket
    #[error("{0} is unknown. Use one of the following: {1}")]
    UnknownPlayer(PlayerId, Participants),
    /// No match to play for player
    #[error("There is no matches for you to play")]
    NoMatchToPlay(Player),
    /// Referred match is unknown
    #[error("Match {0} is unknown")]
    UnknownMatch(MatchId),
    /// Update to match could not happen
    #[error("There is no match to update")]
    NoMatchToUpdate(Vec<Match>, MatchId),
}

/// Returns true if bracket is over
fn bracket_is_over(bracket_matches: &[Match]) -> bool {
    !bracket_matches.iter().any(|m| !m.is_over())
}

/// Returns true when `player_id` has been disqualified by looking into all
/// `matches` in the bracket
fn is_disqualified(player_id: PlayerId, matches: &[Match]) -> bool {
    matches
        .iter()
        .any(|m| m.is_automatic_loser_by_disqualification(player_id))
}

/// Update `bracket` with `updated_match` using a readable one-liner
pub(crate) fn update_bracket_with(bracket: &[Match], updated_match: &Match) -> Vec<Match> {
    bracket
        .iter()
        .cloned()
        .map(|m| {
            if m.get_id() == updated_match.get_id() {
                updated_match.clone()
            } else {
                m
            }
        })
        .collect()
}

/// Updated matches for bracket
///
/// if applicable, the loser of updated match, the expected loser seed that
/// should be used when sending them in lower bracket and a boolean to indicate
/// if the they are disqualified
type BracketUpdate = (Vec<Match>, Option<(Player, usize, bool)>);

/// Takes matches in bracket, validate `match_id` and returns updated winner
/// bracket, id of loser (if there is one), the seed to use when placing them
/// in loser's bracket (if there is one) and whether or not this player is
/// disqualified
///
/// Assuming bracket is always updated after any player is disqualified, there
/// is at most one disqualified player that can drop from winners into loser
///
/// # Errors
/// thrown when attempting an update for winner/loser bracket match in
/// loser/winner bracket
fn update(bracket: &[Match], match_id: MatchId) -> Result<BracketUpdate, Error> {
    let Some(m) = bracket.iter().find(|m| m.get_id() == match_id) else {
        return Err(Error::UnknownMatch(match_id));
    };
    // declare winner if there is one
    let is_disqualified = m.get_automatic_loser() != Opponent::Unknown;
    let (updated_m, winner, loser) = m.clone().update_outcome()?;
    let seed_of_expected_winner = updated_m.get_seeds()[0];
    let expected_loser_seed = updated_m.get_seeds()[1];
    let bracket = update_bracket_with(bracket, &updated_m);

    let last_match = bracket.last().expect("last match in bracket");
    if last_match.get_id() == match_id {
        if Opponent::Unknown != last_match.get_winner() {
            return Ok((bracket, Some((loser, expected_loser_seed, is_disqualified))));
        }
        panic!("No winner of bracket declared when updating bracket finalists match");
    }

    // winner moves forward in bracket
    let index = bracket
        .iter()
        .position(|m| m.get_id() == updated_m.get_id())
        .expect("reference to updated match");
    let mut iter = bracket.iter().skip(index + 1);
    let m = iter
        .find(|m| m.get_seeds().contains(&seed_of_expected_winner))
        .expect("match where winner of updated match plays next");
    let updated_match = m
        .clone()
        .set_player(winner, m.get_seeds()[0] == seed_of_expected_winner);
    let mut bracket = update_bracket_with(&bracket, &updated_match);

    // looser drops to loser bracket in double elimination format

    // Set winner to all matches were a player is disqualified
    // while loop is needed because there can be a scenario where a player
    // is moved up several times because each next match contains a
    // disqualified player
    while bracket
        .iter()
        .any(Match::needs_update_because_of_disqualified_participant)
    {
        let to_update = bracket
            .iter()
            .find(|m| m.needs_update_because_of_disqualified_participant())
            .expect("match with disqualified player");
        let match_id = to_update.get_id();
        let m = bracket
            .iter()
            .find(|m| m.get_id() == match_id)
            .expect("match in bracket");
        let (updated_match, _, _) = m.clone().update_outcome()?;
        bracket = update_bracket_with(&bracket, &updated_match);

        if bracket.last().expect("last match").get_id() == match_id {
            return Ok((bracket, Some((loser, expected_loser_seed, is_disqualified))));
        }

        // winner moves forward in bracket
        let index = bracket
            .iter()
            .position(|m| m.get_id() == updated_match.get_id())
            .expect("reference to updated match");
        let mut iter = bracket.iter().skip(index + 1);
        let seed_of_expected_winner = updated_match.get_seeds()[0];
        let Opponent::Player(winner) = updated_match.get_winner() else {
              panic!("no winner in updated match");  
        };
        let m = iter
            .find(|m| m.get_seeds().contains(&seed_of_expected_winner))
            .expect("match where winner of updated match plays next");
        let updated_match = m
            .clone()
            .set_player(winner.clone(), m.get_seeds()[0] == seed_of_expected_winner);
        bracket = update_bracket_with(&bracket, &updated_match);
    }

    Ok((bracket, Some((loser, expected_loser_seed, is_disqualified))))
}

/// Progression which implements Sized
// pub trait SizedProgression implements Progression + Sized {}

/// Computes the next state of a tournament
pub trait Progression {
    /// Get format of this bracket
    fn get_format(&self) -> Format;

    /// Disqualify participant from bracket and update matches. Returns updated
    /// matches and matches to play
    ///
    /// # Errors
    /// thrown when participant does not belong in tournament
    fn disqualify_participant(
        &self,
        player_id: PlayerId,
    ) -> Result<(Vec<Match>, Vec<Match>), Error>;

    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    fn is_over(&self) -> bool;

    /// List all matches that can be played out
    fn matches_to_play(&self) -> Vec<Match>;

    /// Return next opponent for `player_id`, relevant match and player name
    ///
    /// # Errors
    /// Thrown when matches have yet to be generated or player has won/been
    /// eliminated
    fn next_opponent(&self, player_id: PlayerId) -> Result<(Opponent, MatchId, String), Error>;

    /// Returns true if player is disqualified
    fn is_disqualified(&self, player_id: PlayerId) -> bool;

    /// Report result of match. Returns updated matches, affected match and new
    /// matches to play
    /// # Errors
    /// thrown when player does not belong in bracket
    fn report_result(
        &self,
        player_id: PlayerId,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, MatchId, Vec<Match>), Error>;

    /// Tournament organiser reports result
    /// # Errors
    /// thrown when player does not belong in bracket
    fn tournament_organiser_reports_result(
        &self,
        player1: PlayerId,
        result: (i8, i8),
        player2: PlayerId,
    ) -> Result<(Vec<Match>, MatchId, Vec<Match>), Error>;

    /// Update `match_id` with reported `result` of `player`
    ///
    /// # Errors
    /// thrown when `match_id` matches no existing match
    fn update_player_reported_match_result(
        &self,
        match_id: MatchId,
        result: (i8, i8),
        player_id: PlayerId,
    ) -> Result<Vec<Match>, Error>;

    /// Returns updated matches and matches to play. Uses `match_id` as the
    /// first match to start updating before looking deeper into the bracket
    ///
    /// # Errors
    /// thrown when `match_id` matches no existing match
    fn validate_match_result(&self, match_id: MatchId) -> Result<(Vec<Match>, Vec<Match>), Error>;
}

#[cfg(test)]
fn assert_elimination(s: &dyn Progression, players: &[Player], player_who_won: usize) {
    let iter = players.iter().enumerate();
    let iter = iter.skip(1);

    for (i, player) in iter {
        let e = s
            .next_opponent(player.get_id())
            .expect_err("Eliminated or NoNextMatch");
        if i == player_who_won {
            if let Error::NoNextMatch(eliminated_player) = e {
                assert_eq!(eliminated_player.get_id(), player.get_id());
            } else {
                panic!(
                    "expected Error::Progression(ProgressionError::NoNextMatch error but got {e}"
                );
            }
        } else if let Error::Eliminated(eliminated_player) = e {
            assert_eq!(eliminated_player.get_id(), player.get_id());
        } else {
            panic!("expected Error::Progression(ProgressionError::Eliminated error but got {e:?}");
        }
    }
}

#[cfg(test)]
fn assert_next_matches(
    s: &dyn Progression,
    players_with_unknown_opponent: &[usize],
    expected_matches: &[(usize, usize)],
    players: &[Player],
) {
    for p in players_with_unknown_opponent {
        let player = players[*p].clone();
        let (next_opponent, _, _) = s.next_opponent(player.get_id()).expect("next opponent");
        assert_eq!(
            next_opponent,
            Opponent::Unknown,
            "expected unknown opponent for {p} but got {next_opponent}"
        );
    }

    for (o1, o2) in expected_matches {
        let opponent1 = players[*o1].clone();
        let opponent2 = players[*o2].clone();

        let (next_opponent, _, _) = s.next_opponent(opponent1.get_id()).expect("next opponent");
        if let Opponent::Player(p) = next_opponent {
            assert_eq!(
                p.get_id(),
                opponent2.get_id(),
                "expected {opponent2} for {opponent1} but got {p}"
            );
        } else {
            panic!("expected player for next opponent");
        }
        let (next_opponent, _, _) = s.next_opponent(opponent2.get_id()).expect("next opponent");
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

#[cfg(test)]
/// Assert x wins against y
fn assert_outcome(matches: &[Match], x: &Player, y: &Player) {
    assert!(
        matches
            .iter()
            .any(|m| if m.contains(x.get_id()) && m.contains(y.get_id()) {
                if let Opponent::Player(p) = m.get_winner() {
                    return p.get_id() == x.get_id();
                }
                false
            } else {
                false
            }),
        "No match where {} wins against {}",
        x.get_name(),
        y.get_name()
    );
}

#[cfg(test)]
fn assert_x_wins_against_y(p1: &Player, p2: &Player, matches: &[Match]) {
    assert!(
        matches.iter().any(|m| {
            if let Opponent::Player(winner) = m.get_winner() {
                winner.get_id() == p1.get_id() && m.contains(p2.get_id())
            } else {
                false
            }
        }),
        "no matches where {} wins against {}",
        p1.get_name(),
        p2.get_name()
    );
}
