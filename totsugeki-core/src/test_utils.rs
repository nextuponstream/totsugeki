//! Test utils for unit tests and integration tests

use crate::bracket::seeding::Seeding;
use crate::player::{Participants, Player};
use crate::ID;

/// Test utils for sharing between unit tests and integration tests
#[cfg(test)]
pub struct TestUtils {}

/// All relevant test functions that can be
impl TestUtils {
    /// Returns seeding, a vector with a padded player and an padded vector of
    /// player IDs
    ///
    /// The padded vector allows for more readable access, for example:
    /// players[2] (player 2) vs players[3] (player 3), with respective seeding 2
    /// and 3
    pub fn seeding_for_n_players(
        n: usize,
    ) -> (Seeding, (Vec<Player>, Vec<ID>), (Vec<Player>, Vec<ID>)) {
        let mut padded_players = vec![Player::new("don't use".into())]; // padding for readability
        let mut unpadded_players = vec![]; // padding for readability
        let mut padded_player_ids = vec![ID::default()];
        let mut unpadded_player_ids = vec![];
        let mut seeding = Participants::default();
        for i in 1..=n {
            let player = Player::new(format!("p{i}"));
            padded_player_ids.push(*player.get_id());
            unpadded_player_ids.push(*player.get_id());
            seeding = seeding.add_participant(&player).expect("new participant");
            padded_players.push(player.clone());
            unpadded_players.push(player);
        }
        (
            Seeding::new(unpadded_player_ids.clone()).unwrap(),
            (padded_players, padded_player_ids),
            (unpadded_players, unpadded_player_ids),
        )
    }
}
