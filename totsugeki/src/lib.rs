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

use std::str::FromStr;
use std::sync::{LockResult, RwLock, RwLockReadGuard};
use uuid::Uuid;

pub mod bracket;
pub mod format;
pub mod matches;
pub mod opponent;
pub mod player;
pub mod seeding;

/// Discussion channel identifier
pub type DiscussionChannelId = Uuid;

/// Read-only lock wrapper
pub struct ReadLock<T> {
    // NOTE: needs to be made innaccessible within it's own module so noone can access inner.write()
    /// underlying lock
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
    ///
    /// # Errors
    /// Returns an error if lock is poisoned
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

#[cfg(test)]
/// Helper function for test cases with small group of players. Instead of
/// using Player struct for p1, p2... p9, you can use made up identifiable ids
/// like:
///
/// * 00..01 for player 1
/// * 00..02 for player 2
/// * ... and so on
///
/// # Panics
/// when n is not between 1 and 16
pub(crate) fn legible_uuids_order(n: usize) -> Vec<Uuid> {
    assert_ne!(n, 0, "This function cannot return an empty vector");
    assert!(
        n <= 16,
        "This function does not accept number greater than 9"
    );

    let mut r = vec![];

    for i in 1..=n {
        let id = match i {
            small if small <= 9 => format!("{i:02}"),
            10 => "0A".into(),
            11 => "0B".into(),
            12 => "0C".into(),
            13 => "0D".into(),
            14 => "0E".into(),
            15 => "0F".into(),
            16 => "10".into(),
            _ => unreachable!(),
        };
        let p = format!("00000000-0000-0000-0000-0000000000{id}");
        r.push(p.parse::<crate::player::Id>().expect("id"));
    }

    r
}

#[cfg(test)]
mod tests {
    use crate::legible_uuids_order;

    #[test]
    fn made_up_uuids_work_in_accepted_range() {
        for i in 1..=16 {
            let _uuids = legible_uuids_order(i);
        }
    }
}
