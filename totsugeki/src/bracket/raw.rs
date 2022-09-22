//! Bracket represented in raw strings

use std::fmt::Display;

use super::{Bracket, ParsingError};
use crate::{
    bracket::{http_responses::GET, Id as BracketId},
    format::Format,
    matches::Match,
    player::{Id as PlayerId, Participants, Player},
    seeding::Method as SeedingMethod,
};
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Raw data of bracket, potentially malformed. Use `Bracket` for well-formed bracket
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Raw {
    /// Identifier of this bracket
    pub bracket_id: BracketId,
    /// Name of this bracket
    pub bracket_name: String,
    /// Players in this bracket
    pub players: Vec<PlayerId>,
    /// Names of players in this bracket
    pub player_names: Vec<String>,
    /// Matches from this bracket, sorted by rounds
    pub matches: Vec<Match>,
    /// Bracket format
    pub format: Format,
    /// Seeding method used for this bracket
    pub seeding_method: SeedingMethod,
    /// Advertised start time
    pub start_time: DateTime<Utc>,
    /// Accept match results
    pub accept_match_results: bool,
    /// Matches are automatically validated if both players agree on result
    pub automatic_match_validation: bool,
    /// Bar new participants from entering bracket
    pub barred_from_entering: bool,
}

impl From<Bracket> for Raw {
    fn from(b: Bracket) -> Self {
        Self {
            bracket_id: b.bracket_id,
            bracket_name: b.bracket_name,
            players: b
                .participants
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            player_names: b
                .participants
                .get_players_list()
                .iter()
                .map(Player::get_name)
                .collect(),
            matches: b.matches.clone(),
            format: b.format,
            seeding_method: b.seeding_method,
            start_time: b.start_time,
            accept_match_results: b.accept_match_results,
            automatic_match_validation: b.automatic_match_validation,
            barred_from_entering: b.is_closed,
        }
    }
}

impl TryFrom<Raw> for Bracket {
    type Error = ParsingError;

    fn try_from(br: Raw) -> Result<Self, Self::Error> {
        Ok(Self {
            bracket_id: br.bracket_id,
            bracket_name: br.bracket_name.clone(),
            participants: {
                let players: Vec<(&Uuid, &String)> =
                    br.players.iter().zip(br.player_names.iter()).collect();
                Participants::try_from(players)?
            },
            matches: br.matches,
            format: br.format,
            seeding_method: br.seeding_method,
            start_time: br.start_time,
            accept_match_results: br.accept_match_results,
            automatic_match_validation: br.automatic_match_validation,
            is_closed: br.barred_from_entering,
        })
    }
}

impl Display for Raw {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{ bracket_id: {}, bracket_name \"{} \"}}",
            self.bracket_id, self.bracket_name
        )
    }
}

/// Collection of brackets from raw data
#[derive(Default)]
pub struct Brackets {
    /// A collection of brackets
    brackets: Vec<Raw>,
}

impl Brackets {
    /// Create representation of brackets implementing `std::fmt::Display`
    #[must_use]
    pub fn new(brackets: Vec<Raw>) -> Self {
        Brackets { brackets }
    }

    /// Get brackets
    #[must_use]
    pub fn get_brackets(&self) -> Vec<Raw> {
        self.brackets.clone()
    }
}

impl Display for Brackets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.brackets.clone() {
            b.fmt(f)?;
        }
        Ok(())
    }
}

impl Raw {
    /// Create new bracket
    #[must_use]
    pub fn new(
        bracket_name: String,
        format: Format,
        seeding_method: SeedingMethod,
        start_time: DateTime<Utc>,
        automatic_match_validation: bool,
    ) -> Self {
        Raw {
            bracket_id: BracketId::new_v4(),
            bracket_name,
            players: vec![],
            player_names: vec![],
            matches: vec![],
            format,
            seeding_method,
            start_time,
            accept_match_results: false,
            automatic_match_validation,
            barred_from_entering: false,
        }
    }

    /// Get participants of this bracket as a list of players
    #[must_use]
    pub fn get_players_list(&self) -> Vec<Player> {
        self.players
            .iter()
            .zip(self.player_names.iter())
            .map(|p| Player {
                id: *p.0,
                name: p.1.to_string(),
            })
            .collect()
    }
}

impl TryFrom<GET> for Raw {
    type Error = ParsingError;

    fn try_from(b: GET) -> Result<Self, Self::Error> {
        Ok(Self {
            bracket_id: b.bracket_id,
            bracket_name: b.bracket_name,
            players: b.players.iter().map(Player::get_id).collect(),
            player_names: b.players.iter().map(Player::get_name).collect(),
            matches: b
                .matches
                .into_iter()
                .map(Match::try_from)
                .collect::<Result<Vec<Match>, _>>()?,
            format: b.format.parse::<Format>()?,
            seeding_method: b.seeding_method.parse::<SeedingMethod>()?,
            start_time: b.start_time.parse::<DateTime<Utc>>()?,
            accept_match_results: b.accept_match_results,
            automatic_match_validation: b.automatic_match_validation,
            barred_from_entering: b.barred_from_entering,
        })
    }
}

impl From<Raw> for GET {
    fn from(b: Raw) -> Self {
        GET::new(&b)
    }
}
