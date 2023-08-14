//! Display bracket in any type of frontend
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]
#![forbid(unsafe_code)]

use serde::Serialize;
use totsugeki::matches::{Id as MatchId, Match};
use totsugeki::opponent::Opponent;
use totsugeki::player::Id as PlayerId;
use totsugeki::player::{Participants, Player};

pub mod loser_bracket;
pub mod winner_bracket;

#[derive(Debug, Clone, Serialize)]
/// Strict necessary information to use when displaying a match in UI
pub struct MinimalMatch {
    /// Match identifier
    id: MatchId,
    /// Names of players participating in match
    players: [Player; 2],
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
            players: [
                Player::new(String::default()),
                Player::new(String::default()),
            ],
            score: (0, 0),
            seeds: [0, 0],
            row_hint: None,
        }
    }
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
}

/// Display lines using boxes and their borders
#[derive(Clone, Copy, Debug, PartialEq, Default, Serialize)]
pub struct BoxElement {
    /// true when left border of box should be visible
    pub(crate) left_border: bool,
    /// true when bottom border of box should be visible
    pub(crate) bottom_border: bool,
}

impl BoxElement {
    /// Box with no borders. Alternative to `default()` to use in constants
    const fn empty() -> Self {
        BoxElement {
            left_border: false,
            bottom_border: false,
        }
    }
}

/// Convert match struct from Totsugeki library into minimal struct, using
/// `participants` to fill in name of players.
#[must_use]
pub fn from_participants(m: &Match, participants: &Participants) -> MinimalMatch {
    let list = participants.get_players_list();
    let players: Vec<(PlayerId, String)> =
        list.iter().map(|p| (p.get_id(), p.get_name())).collect();

    // TODO find out if storing both player name and id is better than storing
    // only the id and doing some work to get back id and name.
    let p1 = match m.get_players()[0] {
        Opponent::Player(id) => id,
        Opponent::Unknown => PlayerId::new_v4(),
    };
    let p2 = match m.get_players()[1] {
        Opponent::Player(id) => id,
        Opponent::Unknown => PlayerId::new_v4(),
    };
    let top_seed = m.get_players()[0].get_name(&players);
    let bottom_seed = m.get_players()[1].get_name(&players);
    MinimalMatch {
        id: m.get_id(),
        players: [
            Player::from((p1, top_seed)),
            Player::from((p2, bottom_seed)),
        ],
        score: m.get_score(),
        seeds: m.get_seeds(),
        row_hint: None,
    }
}
