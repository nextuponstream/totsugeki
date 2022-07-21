#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

use bracket::{ActiveBrackets, FinalizedBrackets};
//use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{LockResult, RwLock, RwLockReadGuard};
use std::{collections::HashMap, fmt::Display};
use uuid::Uuid;

pub mod bracket;
pub mod organiser;

#[derive(Serialize, Deserialize)]
/// Body of bracket POST request
pub struct BracketPOST {
    /// name of the bracket
    pub bracket_name: String,
}

impl BracketPOST {
    /// Create new Bracket POST request
    #[must_use]
    pub fn new(bracket_name: String) -> Self {
        BracketPOST { bracket_name }
    }
}

/// Bracket for a tournament
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Bracket {
    id: OrganiserId,
    bracket_name: String,
}

impl Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{ id: {}, bracket_name \"{} \"}}",
            self.id, self.bracket_name
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
    pub fn new(id: OrganiserId, bracket_name: String) -> Self {
        // TODO add check where registration_start_time < beginning_start_time
        Bracket { id, bracket_name }
    }

    /// Get ID of bracket
    #[must_use]
    pub fn get_id(&self) -> OrganiserId {
        self.id
    }

    /// Get name of bracket
    #[must_use]
    pub fn get_bracket_name(self) -> String {
        self.bracket_name
    }
}

/// Bracket identifier
pub type BracketId = Uuid;

/// Discussion channel identifier
pub type DiscussionChannelId = Uuid;

/// Organiser identifier
pub type OrganiserId = Uuid;

/// Tournament organiser with TO's runnning brackets
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Organiser {
    organiser_id: OrganiserId,
    organiser_name: String,
    active_brackets: ActiveBrackets,
    finalized_brackets: FinalizedBrackets,
    // TODO location type
}

impl Organiser {
    /// Create new tournament organiser
    pub fn new(name: String) -> Self {
        Self {
            organiser_id: Uuid::new_v4(),
            organiser_name: name,
            active_brackets: HashMap::new(),
            finalized_brackets: HashMap::new(),
        }
    }

    #[must_use]
    /// Get organiser id
    pub fn get_organiser_id(&self) -> OrganiserId {
        self.organiser_id
    }

    #[must_use]
    /// Get organiser name
    pub fn get_organiser_name(&self) -> String {
        self.organiser_name.clone()
    }

    #[must_use]
    /// Get active brackets
    pub fn get_active_brackets(&self) -> ActiveBrackets {
        self.active_brackets.clone()
    }

    #[must_use]
    /// Get finalized brackets
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

/// Read-only lock wrapper
pub struct ReadLock<T> {
    // NOTE: needs to be made innaccessible within it's own module so noone can access inner.write()
    inner: RwLock<T>,
}

impl<T> ReadLock<T> {
    /// Create new read-only guard
    pub fn new(t: T) -> Self {
        Self {
            inner: RwLock::new(t),
        }
    }

    /// Give read handle over ressource
    pub fn read(&self) -> LockResult<RwLockReadGuard<'_, T>> {
        self.inner.read()
    }
}

/// Id of service
pub type ServiceId = Uuid;

/// Discussion channel
pub trait DiscussionChannel {
    /// Type of internal id
    type InternalId: FromStr + ToString;

    /// Get channel id
    fn get_channel_id(&self) -> Option<DiscussionChannelId>;

    /// Get internal id
    fn get_internal_id(&self) -> Self::InternalId;

    /// Get type of service
    fn get_service_type(&self) -> String;
}

/// Response body
#[derive(Deserialize)]
pub struct ServiceRegisterPOST {
    id: ServiceId,
    token: String,
}

impl ServiceRegisterPOST {
    #[must_use]
    /// Get id of service
    pub fn get_id(&self) -> ServiceId {
        self.id
    }

    #[must_use]
    /// Get authorization header for API
    pub fn get_token(&self) -> String {
        self.token.clone()
    }
}
