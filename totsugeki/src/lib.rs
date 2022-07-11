#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

//use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::sync::{LockResult, RwLock, RwLockReadGuard};
use std::{collections::HashMap, fmt::Display};
use uuid::Uuid;

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
    id: i64, // TODO change to UUID
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
    pub fn new(id: i64, bracket_name: String) -> Self {
        // TODO add check where registration_start_time < beginning_start_time
        Bracket { id, bracket_name }
    }

    /// Get ID of bracket
    #[must_use]
    pub fn get_id(&self) -> i64 {
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

/// Tournament organiser with TO's runnning brackets
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Organiser {
    uuid: Uuid,
    name: String,
    active_brackets: HashMap<DiscussionChannelId, BracketId>,
    finalized_brackets: HashMap<BracketId, Bracket>,
    // TODO location type
}

impl Organiser {
    /// Create new tournament organiser
    pub fn new(name: String) -> Self {
        Self {
            uuid: Uuid::new_v4(),
            name,
            active_brackets: HashMap::new(),
            finalized_brackets: HashMap::new(),
        }
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
