#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

use bracket::ActiveBrackets;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::sync::{LockResult, RwLock, RwLockReadGuard};
use uuid::Uuid;

pub mod bracket;
pub mod join;
pub mod organiser;

/// Discussion channel identifier
pub type DiscussionChannelId = Uuid;

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

/// Player identifier
pub type PlayerId = Uuid;
