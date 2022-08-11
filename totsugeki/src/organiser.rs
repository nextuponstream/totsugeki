//! Organiser domain

use crate::{bracket::Id as BracketId, ActiveBrackets};
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use uuid::Uuid;

/// Organiser identifier
pub type Id = Uuid;

/// Brackets where players have finished playing
type FinalizedBrackets = HashSet<BracketId>;

/// Organiser of events
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Organiser {
    /// Active brackets from organiser
    active_brackets: ActiveBrackets,
    /// Finalized brackets from organiser
    finalized_brackets: FinalizedBrackets,
    /// Identifier of organiser
    organiser_id: Id,
    /// Name of the organiser
    organiser_name: String,
    // TODO location type
}

impl Organiser {
    /// Create new organiser of events
    #[must_use]
    pub fn new(
        organiser_id: Id,
        organiser_name: String,
        active_brackets: Option<ActiveBrackets>,
    ) -> Self {
        Self {
            organiser_id,
            organiser_name,
            active_brackets: if let Some(a) = active_brackets {
                a
            } else {
                HashMap::new()
            },
            finalized_brackets: HashSet::new(),
        }
    }

    #[must_use]
    /// Create organiser from values
    pub fn from(
        active_brackets: ActiveBrackets,
        finalized_brackets: FinalizedBrackets,
        organiser_id: Id,
        organiser_name: String,
    ) -> Self {
        Self {
            active_brackets,
            finalized_brackets,
            organiser_id,
            organiser_name,
        }
    }

    #[must_use]
    /// Get UUID of organiser
    pub fn get_organiser_id(&self) -> Id {
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

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
/// Organiser POST request body
pub struct POSTRequest {
    /// Name of the organiser to create
    pub organiser_name: String,
}

#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
/// Organiser GET response
pub struct GETResponse {
    /// Identifier of the organiser
    pub organiser_id: Id,
    /// Name of the organiser
    pub organiser_name: String,
    /// Active bracket managed by this organiser
    pub active_brackets: ActiveBrackets,
    /// Finalized bracket from this organiser
    pub finalized_brackets: FinalizedBrackets,
}

impl From<Organiser> for GETResponse {
    fn from(o: Organiser) -> Self {
        Self {
            organiser_id: o.get_organiser_id(),
            organiser_name: o.get_organiser_name(),
            active_brackets: o.get_active_brackets(),
            finalized_brackets: o.get_finalized_brackets(),
        }
    }
}
