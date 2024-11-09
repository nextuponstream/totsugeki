//! Check next opponent

use crate::matches::MatchID;
use crate::opponent::Opponent;
use crate::player::PlayerID;
use crate::ID;
use thiserror::Error;

/// For all formats that have a bracket, query next opponent through consistent interface
pub trait NextOpponentInBracket {
    /// Return next opponent for `player_id` if any and relevant match ID
    fn next_opponent_in_bracket(&self, player_id: PlayerID) -> Result<(Opponent, MatchID), Error>;
}

#[derive(Error, Debug)]
/// There is no next opponent for player for a given reason
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
