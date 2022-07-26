//! Bracket object

use crate::{organiser::OrganiserId, DiscussionChannelId, PlayerId};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Display};
use uuid::Uuid;

/// Active brackets
pub type ActiveBrackets = HashMap<DiscussionChannelId, BracketId>;

/// Finalized brackets
pub type FinalizedBrackets = HashMap<BracketId, Bracket>;

#[derive(Serialize, Deserialize)]
/// Body of bracket POST request
pub struct BracketPOST {
    /// name of the bracket
    bracket_name: String,
    /// used to create missing organiser
    organiser_name: String,
    organiser_internal_id: String,
    channel_internal_id: String,
    service_type_id: String,
}

impl BracketPOST {
    /// Create new Bracket POST request
    #[must_use]
    pub fn new(
        bracket_name: String,
        organiser_name: String,
        organiser_internal_id: String,
        channel_internal_id: String,
        service_type_id: String,
    ) -> Self {
        BracketPOST {
            bracket_name,
            organiser_name,
            organiser_internal_id,
            channel_internal_id,
            service_type_id,
        }
    }
}

/// Bracket for a tournament
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Bracket {
    bracket_id: BracketId,
    bracket_name: String,
    players: Vec<PlayerId>,
}

impl Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{ bracket_id: {}, bracket_name \"{} \"}}",
            self.bracket_id, self.bracket_name
        )
    }
}

/// A collection of brackets
#[derive(Default)]
pub struct Brackets {
    brackets: Vec<Bracket>,
}

impl Brackets {
    /// Create representation of brackets implementing `std::fmt::Display`
    #[must_use]
    pub fn new(brackets: Vec<Bracket>) -> Self {
        Brackets { brackets }
    }

    /// Get brackets
    #[must_use]
    pub fn get_brackets(&self) -> Vec<Bracket> {
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

impl Bracket {
    /// Create new bracket
    #[must_use]
    pub fn new(bracket_name: String, players: Vec<PlayerId>) -> Self {
        // TODO add check where registration_start_time < beginning_start_time
        Bracket {
            bracket_id: Uuid::new_v4(),
            bracket_name,
            players,
        }
    }

    /// Create from existing bracket
    #[must_use]
    pub fn from(id: BracketId, bracket_name: String, players: Vec<PlayerId>) -> Self {
        Self {
            bracket_id: id,
            bracket_name,
            players,
        }
    }

    /// Get ID of bracket
    #[must_use]
    pub fn get_id(&self) -> Uuid {
        self.bracket_id
    }

    /// Get name of bracket
    #[must_use]
    pub fn get_bracket_name(&self) -> String {
        self.bracket_name.clone()
    }

    /// Get players
    #[must_use]
    pub fn get_players(&self) -> Vec<PlayerId> {
        self.players.clone()
    }
}

/// Bracket identifier
pub type BracketId = Uuid;

/// Response to Bracket POST request
#[derive(Serialize, Deserialize)]
pub struct BracketPOSTResult {
    /// id of created bracket
    bracket_id: BracketId,
    /// id of organiser
    organiser_id: OrganiserId,
    /// id of discussion channel
    discussion_channel_id: DiscussionChannelId,
}

impl BracketPOSTResult {
    #[must_use]
    /// Create new bracket from values
    pub fn from(
        bracket_id: BracketId,
        organiser_id: OrganiserId,
        discussion_channel_id: DiscussionChannelId,
    ) -> Self {
        Self {
            bracket_id,
            organiser_id,
            discussion_channel_id,
        }
    }

    #[must_use]
    /// Get bracket id
    pub fn get_bracket_id(&self) -> BracketId {
        self.bracket_id
    }

    #[must_use]
    /// Get organiser id
    pub fn get_organiser_id(&self) -> OrganiserId {
        self.organiser_id
    }

    #[must_use]
    /// Get discussion channel id
    pub fn get_discussion_channel_id(&self) -> DiscussionChannelId {
        self.discussion_channel_id
    }
}
