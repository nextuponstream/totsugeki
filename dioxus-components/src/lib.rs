//! dioxus UI components and functions to display a bracket

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]
#![warn(clippy::unwrap_used)]

use crate::components::bracket::displayable_match::EMPTY_NAME;
use totsugeki::{
    matches::{Id as MatchId, Match},
    player::{Id as PlayerId, Participants},
};

pub mod components;
pub mod ordering;

/// Maximum size for name
const MAX_NAME_SIZE: usize = 64;
/// Name that can be copied over
type Name = [u8; MAX_NAME_SIZE];

#[derive(Debug, Clone, Copy)]
/// Strict necessary information to use when displaying a match in UI
pub struct MinimalMatch {
    /// Match identifier
    id: MatchId,
    /// Names of players participating in match
    pub(crate) players: [Name; 2],
    /// Score of match
    score: (i8, i8),
    /// Expected seeds of player in match
    seeds: [usize; 2],
    /// Indicate which row it belongs to, starting from 0 index
    row_hint: Option<usize>,
}

impl Default for MinimalMatch {
    fn default() -> Self {
        MinimalMatch {
            id: MatchId::new_v4(),
            players: [ShortName::default().value, ShortName::default().value],
            score: (0, 0),
            seeds: [0, 0],
            row_hint: None,
        }
    }
}

// impl std::fmt::Debug for DisplayableMatch {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{:?} {:?}", self.seeds, self.row_hint)
//     }
// }

/// Display minimal match errors
enum Error {
    /// Could not display player name
    PlayerName,
}

impl MinimalMatch {
    #[cfg(test)]
    fn new(seeds: [usize; 2]) -> Self {
        Self {
            seeds,
            ..Self::default()
        }
    }

    #[cfg(test)]
    fn summary(&self) -> String {
        format!("{:?}; row hint = {:?}", self.seeds, self.row_hint)
    }

    /// Returns name of player when `is_player1` is true, player 2 otherwise
    fn player(&self, is_player1: bool) -> Result<&str, Error> {
        let id = if is_player1 { 0 } else { 1 };
        let Ok(name) = std::str::from_utf8(&self.players[id]) else {
            return Err(Error::PlayerName);
        };
        Ok(name)
    }

    /// Returns name of player 1
    fn player1(&self) -> &str {
        match self.player(true) {
            Ok(name) => name,
            Err(_e) => {
                // TODO log error
                "err"
            }
        }
    }

    /// Returns name of player 2
    fn player2(&self) -> &str {
        match self.player(false) {
            Ok(name) => name,
            Err(_e) => {
                // TODO log error
                "err"
            }
        }
    }

    /// Returns score of player 1
    fn score1(&self) -> String {
        self.score.0.to_string()
    }

    /// Returns score of player 2
    fn score2(&self) -> String {
        self.score.1.to_string()
    }
}

/// A struct to represent names in UI that contains maximum `MAX_NAME_SIZE`
/// caracters
struct ShortName {
    /// Name as bytes
    pub(crate) value: [u8; MAX_NAME_SIZE],
}

impl ShortName {
    /// Get utf8 string for name
    pub(crate) fn get(&self) -> String {
        // TODO remove expect
        String::from_utf8(self.value.into()).expect("string")
    }
}

impl std::default::Default for ShortName {
    fn default() -> Self {
        let mut value = String::default()
            .into_bytes()
            .into_iter()
            .collect::<Vec<u8>>();
        value.resize(MAX_NAME_SIZE, 0);
        // let value = value.try_into().unwrap();
        let value = EMPTY_NAME;
        Self { value }
    }
}

/// Convert match struct from Totsugeki library into minimal struct, using
/// `participants` to fill in name of players.
///
///
fn from_participants(m: &Match, participants: &Participants) -> MinimalMatch {
    let list = participants.get_players_list();
    let players: Vec<(PlayerId, String)> =
        list.iter().map(|p| (p.get_id(), p.get_name())).collect();
    let player1 = convert_to_displayable_name(m.get_players()[0].get_name(&players));
    let player2 = convert_to_displayable_name(m.get_players()[1].get_name(&players));
    MinimalMatch {
        id: m.get_id(),
        players: [player1, player2],
        score: m.get_score(),
        seeds: m.get_seeds(),
        row_hint: None,
    }
}

/// Converts name of player of totsugeki library into displayable name of fixed
/// size `MAX_NAME_SIZE` to pass around as argument
pub(crate) fn convert_to_displayable_name(name: String) -> Name {
    let mut name = name
        .into_bytes()
        .into_iter()
        .take(MAX_NAME_SIZE)
        .collect::<Vec<u8>>();
    name.resize(MAX_NAME_SIZE, 0); // '\0' null byte
    match name.try_into() {
        Ok(n) => n,
        Err(_e) => {
            // TODO log error
            EMPTY_NAME
        }
    }
}

/// All types of modals necessary for updating a bracket
pub enum Modal {
    /// Add player to bracket
    AddPlayer,
    /// Enter result for given `MatchId` between player 1 and player 2
    EnterMatchResult(MatchId, Name, Name),
    /// Disqualify player from bracket
    Disqualify,
}

#[cfg(test)]
mod test {
    use crate::ShortName;

    #[test]
    fn get_default_short_name() {
        ShortName::default();
    }
}
