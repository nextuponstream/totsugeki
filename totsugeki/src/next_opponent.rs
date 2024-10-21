//! Check next opponent

use crate::opponent::Opponent;
use crate::ID;

/// For all formats that have a bracket, query next opponent through consistent interface
pub trait NextOpponentInBracket {
    /// Return next opponent for `player_id` if any and relevant match ID
    fn next_opponent_in_bracket(&self, player_id: ID) -> Option<(Option<Opponent>, ID)>;
}

/// For all formats that have a group stage, query next opponent through consistent interface
#[allow(unused)]
trait NextOpponentInGroupStage {
    /// Return all known future opponents for `player_id` and relevant match IDs
    fn next_opponent_in_pool(&self, player_id: ID) -> Vec<Option<(Option<Opponent>, ID)>>;
}
