//! Check next opponent

use crate::matches::MatchID;
use crate::opponent::Opponent;
use crate::player::PlayerID;
use crate::ID;
use thiserror::Error;

/// For all formats that have a bracket, query next opponent through consistent interface
#[allow(clippy::module_name_repetitions)]
pub trait NextOpponentInBracket {
    /// Return next opponent for `player_id` if any and relevant match ID
    ///
    /// # Errors
    /// Given a certain state of the bracket, this player will may not have
    /// opponent even if the bracket progresses further
    fn next_opponent_in_bracket(&self, player_id: PlayerID) -> Result<(Opponent, MatchID), Error>;
}

#[derive(Error, Debug)]
/// Given a certain state of the bracket, this player will may not have
/// opponent even if the bracket progresses further
pub enum Error {
    /// Player has won the bracket
    #[error("Player has won bracket")]
    TournamentWon,
    /// Player is eliminated from bracket
    #[error("Player is eliminated from bracket")]
    Eliminated,
}

/// For all formats that have a group stage, query next opponent through consistent interface
#[allow(unused)]
trait NextOpponentInGroupStage {
    /// Return all known future opponents for `player_id` and relevant match IDs
    fn next_opponent_in_pool(&self, player_id: ID) -> Vec<Option<(Option<Opponent>, ID)>>;
}
