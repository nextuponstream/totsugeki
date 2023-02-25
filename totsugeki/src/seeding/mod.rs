//! Seed brackets with seeding methods

pub mod double_elimination_seeded_bracket;
pub mod single_elimination_seeded_bracket;

use crate::{
    matches::Match,
    opponent::Opponent,
    player::{Error as PlayerError, Id as PlayerId, Participants},
};
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use rand::prelude::*;
use rand::rngs::OsRng;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Seeding method
#[derive(Copy, Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub enum Method {
    /// Randomize who plays against who
    Random,
    /// Sort players by perceived strength to avoid pitting them against each
    /// other early in the bracket
    Strict,
}

impl std::fmt::Display for Method {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Method::Random => write!(f, "random"),
            Method::Strict => write!(f, "strict"),
        }
    }
}

/// Seeding method parsing error
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Unknown seeding method was found
    #[error("Seeding method is unknown")]
    Unknown,
}

impl std::str::FromStr for Method {
    type Err = ParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "random" => Self::Random,
            "strict" => Self::Strict,
            _ => return Err(ParsingError::Unknown),
        })
    }
}

impl Default for Method {
    fn default() -> Self {
        Self::Strict
    }
}

/// Seeding cannot proceed
#[derive(Error, Debug)]
pub enum Error {
    /// You cannot seed a bracket with less than 3 players
    #[error("Not enough players")]
    NotEnoughPlayers,
    /// The os generator panicked while generating a random number
    #[error("RNG is unavailable")]
    Rng(#[from] rand::Error),
    /// Shuffle could not yield players
    #[error("A shuffling operation could not be performed: {0}")]
    Shuffle(#[from] PlayerError),
    /// Mathematical overflow
    #[error("A mathematical overflow happened")]
    MathOverflow,
    /// Seeding needs to use the same participants
    #[error("Participants used for seeding differ from current participants:\nused: {0}\nactual participants: {1}")]
    DifferentParticipants(Participants, Participants),
}

/// Returns updated participants after changing seeding position
///
/// With `Strict` method, `players` are expected to be ranked from strongest to
/// weakest.
///
/// # Errors
/// Returns an error when filling an empty bracket or group of players cannot
/// be formed
pub fn seed(
    method: &Method,
    seeding: Participants,
    participants: Participants,
) -> Result<Participants, Error> {
    if participants.len() < 3 {
        return Err(Error::NotEnoughPlayers);
    }

    match method {
        Method::Random => {
            let mut key = [0u8; 16];
            OsRng.try_fill_bytes(&mut key)?;
            let mut rng = OsRng::default();
            let mut players = participants.get_players_list();
            players.shuffle(&mut rng);
            let players = Participants::try_from(players)?;
            Ok(players)
        }
        Method::Strict => {
            if participants.have_same_participants(&seeding) {
                Ok(seeding)
            } else {
                Err(Error::DifferentParticipants(seeding, participants))
            }
        }
    }
}

/// Pushes one seeded match matching top seed and bottom seed onto
/// `this_round`. Because it's the initial round where either top seed is
/// present (example: 8 man bracket) or they are not (3 man bracket), then the
/// number of available players is a multiple of two.
fn seeding_initial_round(
    available_players_by_seeds: &mut Vec<usize>,
    seeding: &[PlayerId],
    this_round: &mut Vec<Match>,
) {
    let top_seed = available_players_by_seeds.remove(0);
    let top_seed_player_id = seeding[top_seed - 1];
    let bottom_seed = available_players_by_seeds.pop().expect("bottom seed");
    let bottom_seed_player_id = seeding[bottom_seed - 1];

    this_round.push(
        Match::new(
            [
                Opponent::Player(top_seed_player_id),
                Opponent::Player(bottom_seed_player_id),
            ],
            [top_seed, bottom_seed],
        )
        .expect("match"),
    );
}

/// Request to seed a bracket
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
#[cfg_attr(feature = "poem-openapi", oai(rename = "SeedingPOST"))]
pub struct POST {
    /// Discussion channel internal id
    pub internal_channel_id: String,
    /// Service
    pub service: String,
    /// List of seeded players
    pub players: Vec<String>,
}

#[cfg(test)]
mod tests {
    use crate::player::{Participants, Player};
    use crate::seeding::{seed, Error, Method};

    fn assert_seeding_returns_not_enough_player_error(
        players: Participants,
        current_participants: Participants,
    ) {
        match seed(&Method::Random, players, current_participants) {
            Err(Error::NotEnoughPlayers) => {}
            Err(e) => panic!("expected error but got {e}"),
            Ok(_) => panic!("expected error but got none"),
        }
    }

    #[test]
    fn cannot_seed_bracket_without_3_players_minimum() {
        let mut players = Participants::default();
        assert_seeding_returns_not_enough_player_error(players.clone(), players.clone());

        players = players
            .add_participant(Player::new("player1".to_string()))
            .expect("player added");
        assert_seeding_returns_not_enough_player_error(players.clone(), players.clone());

        players = players
            .add_participant(Player::new("player2".to_string()))
            .expect("player added");
        assert_seeding_returns_not_enough_player_error(players.clone(), players.clone());

        assert_eq!(
            players.len(),
            2,
            "there should be two players, found: {}",
            players.len()
        );
    }
}
