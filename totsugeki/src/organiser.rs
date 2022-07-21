//! Organiser domain

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    bracket::{Bracket, BracketId},
    ActiveBrackets,
};

/// Organiser identifier
pub type OrganiserId = Uuid;

type FinalizedBrackets = HashMap<BracketId, Bracket>;

/// Organiser of events
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Organiser {
    active_brackets: ActiveBrackets,
    finalized_brackets: FinalizedBrackets,
    organiser_id: OrganiserId,
    organiser_name: String,
    // TODO location type
}

impl Organiser {
    /// Create new organiser of events
    pub fn new(organiser_id: Uuid, name: String, active_brackets: Option<ActiveBrackets>) -> Self {
        Self {
            organiser_id,
            organiser_name: name,
            active_brackets: if let Some(a) = active_brackets {
                a
            } else {
                HashMap::new()
            },
            finalized_brackets: HashMap::new(),
        }
    }

    #[must_use]
    /// Get UUID of organiser
    pub fn get_organiser_id(&self) -> OrganiserId {
        self.organiser_id
    }

    #[must_use]
    /// Get name of organiser
    pub fn get_organiser_name(&self) -> String {
        self.organiser_name.clone()
    }

    #[must_use]
    /// Get active brackets
    pub fn get_active_brackets(&self) -> ActiveBrackets {
        self.active_brackets.clone()
    }

    #[must_use]
    /// Get active brackets
    pub fn get_finalized_brackets(&self) -> FinalizedBrackets {
        self.finalized_brackets.clone()
    }
}

#[derive(Serialize, Deserialize)]
/// Body of organiser POST request
pub struct OrganiserPOST {
    /// name of the organiser
    pub organiser_name: String,
}

impl OrganiserPOST {
    /// Create new Organiser POST request
    #[must_use]
    pub fn new(organiser_name: String) -> Self {
        OrganiserPOST { organiser_name }
    }
}
