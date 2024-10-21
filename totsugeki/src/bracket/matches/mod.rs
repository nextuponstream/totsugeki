//! Manage matches from bracket

use crate::matches::GenerationError;
#[cfg(test)]
use crate::player::Player;
use crate::{
    matches::{Error as MatchError, Id as MatchId, Match},
    opponent::Opponent,
    player::Id as PlayerId,
    seeding::Error as SeedingError,
    ID,
};
use std::fmt::format;
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
    #[error("{0} is disqualified")]
    Disqualified(PlayerId),
    /// Player has won the tournament and has no match left to play
    #[error("{0} won the tournament and has no matches left to play")]
    NoNextMatch(PlayerId),
    /// Player is eliminated from tournament and has no matches left to play
    #[error("{0} has been eliminated from the tournament and has no matches left to play")]
    Eliminated(PlayerId),
    /// Tournament is over
    #[error("Tournament is over")]
    TournamentIsOver,
    /// Cannot update match
    #[error("{0}")]
    MatchUpdate(#[from] MatchError),
    /// Player is unknown in this bracket
    #[error("{0} is unknown. Use one of the following: {1:?}")]
    UnknownPlayer(PlayerId, Vec<PlayerId>),
    /// No match to play for player
    #[error("There is no matches for you to play")]
    NoMatchToPlay(PlayerId),
    /// Referred match is unknown
    #[error("Match {0} is unknown")]
    UnknownMatch(MatchId),
    /// Update to match could not happen
    #[error("There is no match to update")]
    NoMatchToUpdate(Vec<Match>, MatchId),
    /// Fordidden action because player has been disqualified
    #[error("{0} is disqualified")]
    ForbiddenDisqualified(PlayerId),
}

// FIXME remove
impl From<GenerationError> for Error {
    fn from(value: GenerationError) -> Self {
        Self::MatchUpdate(MatchError::SamePlayer)
    }
}

/// Returns true if bracket is over
pub(crate) fn bracket_is_over(bracket_matches: &[Match]) -> bool {
    !bracket_matches.iter().any(|m| !m.is_over())
}

/// Returns true when `player_id` has been disqualified by looking into all
/// `matches` in the bracket
pub(crate) fn is_disqualified(player_id: PlayerId, matches: &[Match]) -> bool {
    matches
        .iter()
        .any(|m| m.is_automatic_loser_by_disqualification(player_id))
}

/// Update `bracket` with `updated_match` using a readable one-liner
pub(crate) fn update_bracket_with(bracket: &[Match], updated_match: &Match) -> Vec<Match> {
    bracket
        .iter()
        .copied()
        .map(|m| {
            if m.get_id() == updated_match.get_id() {
                *updated_match
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
/// if they are disqualified
type BracketUpdate = (Vec<Match>, Option<(PlayerId, usize, bool)>);

// FIXME should be made on self and consumed
/// Takes matches in bracket, validate `match_id` and returns updated winner
/// bracket, id of loser (if there is one), the seed to use when placing them
/// in loser's bracket (if there is one) and whether this player is disqualified
///
/// Assuming bracket is always updated after any player is disqualified, there
/// is at most one disqualified player that can drop from winners into loser
///
/// # Errors
/// thrown when attempting an update for winner/loser bracket match in
/// loser/winner bracket
pub(crate) fn update(bracket_matches: &[Match], match_id: MatchId) -> Result<BracketUpdate, Error> {
    let m = bracket_matches
        .iter()
        .find(|m| m.get_id() == match_id)
        .expect(format!("match {} updated", match_id).as_str());
    // declare winner if there is one
    let is_disqualified = m.get_automatic_loser() != Opponent::Unknown;
    let (updated_m, winner, loser) = (*m).update_outcome()?;
    let seed_of_expected_winner = updated_m.get_seeds()[0];
    let expected_loser_seed = updated_m.get_seeds()[1];
    let bracket = update_bracket_with(bracket_matches, &updated_m);

    let last_match = bracket.last().expect("last match in bracket");
    match (last_match.get_id(), last_match.get_winner()) {
        (id, Opponent::Player(_)) if id == match_id => {
            return Ok((bracket, Some((loser, expected_loser_seed, is_disqualified))))
        }
        (id, Opponent::Unknown) if id == match_id => {
            panic!("No winner of bracket declared when updating bracket finalists match")
        }
        _ => {}
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
    let updated_match = (*m).insert_player(winner, m.get_seeds()[0] == seed_of_expected_winner);
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
        let (updated_match, _, _) = (*m).update_outcome()?;
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
            panic!(
                "no winner in updated match. Corrupted data for updated match {:?}",
                updated_match
            );
        };
        let m = iter
            .find(|m| m.get_seeds().contains(&seed_of_expected_winner))
            .expect("match where winner of updated match plays next");
        let updated_match = (*m).insert_player(winner, m.get_seeds()[0] == seed_of_expected_winner);
        bracket = update_bracket_with(&bracket, &updated_match);
    }

    Ok((bracket, Some((loser, expected_loser_seed, is_disqualified))))
}

/// Assert any players set as disqualified at most once
pub(crate) fn assert_disqualified_at_most_once(matches: &[Match], seeding: &[PlayerId]) {
    for player in seeding {
        assert!(
            matches
                .iter()
                .filter(
                    |m| matches!(m.get_automatic_loser(), Opponent::Player(id) if id == *player)
                )
                .count()
                < 2
        );
    }
}

/// Assert if both opponent are not the same player
pub(crate) fn assert_match_is_well_formed(m: &Match) {
    assert!(
        !matches!(m.get_players(), [Opponent::Player(p1), Opponent::Player(p2)] if p1 == p2),
        "match {m} is not well formed"
    );
}

/// Computes the next state of a tournament
pub trait Progression {
    /// Disqualify participant from bracket and update matches. Returns updated
    /// matches and matches to play
    ///
    /// # Errors
    /// thrown when participant does not belong in tournament
    fn disqualify_participant(&self, player_id: ID) -> Result<(Vec<Match>, Vec<Match>), Error>;

    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    fn is_over(&self) -> bool;

    /// Returns true if bracket is over (all matches are played)
    #[must_use]
    fn matches_progress(&self) -> (usize, usize);

    /// List all matches that can be played out
    fn matches_to_play(&self) -> Vec<Match>;

    /// Return next opponent for `player_id`, relevant match and player name
    ///
    /// # Errors
    /// Thrown when matches have yet to be generated or player has won/been
    /// eliminated
    fn next_opponent(&self, player_id: PlayerId) -> Result<(Opponent, MatchId), Error>;

    /// Returns true if player is disqualified
    fn is_disqualified(&self, player_id: PlayerId) -> bool;

    /// Report result of match. Returns updated matches, affected match and new
    /// matches to play
    /// # Errors
    /// thrown when player does not belong in bracket
    /// # Panics
    /// When player does not belong in bracket
    fn report_result(
        &self,
        player_id: PlayerId,
        result: (i8, i8),
    ) -> Result<(Vec<Match>, MatchId, Vec<Match>), Error>;

    /// Tournament organiser reports result
    ///
    /// NOTE: both players are needed, so it is less ambiguous when reading code:
    /// * p1 2-0 is more ambiguous to read than
    /// * p1 2-0 p2
    ///
    /// Technically, it's unnecessary.
    ///
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

    /// Checks all assertions after updating matches
    fn check_all_assertions(&self);
}

#[cfg(test)]
pub(crate) fn assert_elimination(s: &dyn Progression, players: &[Player], player_who_won: usize) {
    let iter = players.iter().enumerate();
    let iter = iter.skip(1);

    for (i, player) in iter {
        match (i, s.next_opponent(player.get_id())) {
            (i, Err(Error::NoNextMatch(eliminated_player))) if i == player_who_won => {
                assert_eq!(eliminated_player, player.get_id());
            }
            (i, e) if i == player_who_won => {
                panic!(
                    "expected {:?} but got {e:?}",
                    Error::NoNextMatch(player.get_id())
                )
            }
            (_, Err(Error::Eliminated(eliminated_player))) => {
                assert_eq!(eliminated_player, player.get_id());
            }
            (_, e) => panic!(
                "expected {:?} but got {e:?}",
                Error::Eliminated(player.get_id())
            ),
        }
    }
}

#[cfg(test)]
/// Assert x wins against y
fn assert_outcome(matches: &[Match], x: &Player, y: &Player) {
    assert!(
        matches.iter().any(|m| matches!((
                m.contains(x.get_id()),
                m.contains(y.get_id()),
                m.get_winner()
            ), (true, true, Opponent::Player(winner)) if winner == x.get_id())),
        "No match where {} wins against {}",
        x.get_name(),
        y.get_name()
    );
}

#[cfg(test)]
fn assert_x_wins_against_y(p1: &Player, p2: &Player, matches: &[Match]) {
    assert!(
        matches.iter().any(|m| {
            matches!((m.get_winner(), m.contains(p2.get_id())), (Opponent::Player(winner), true) if winner == p1.get_id())
        }),
        "no matches where {} wins against {}",
        p1.get_name(),
        p2.get_name()
    );
}
