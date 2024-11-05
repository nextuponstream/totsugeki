//! Builer pattern for brackets

use crate::bracket::Bracket;
use crate::{format::Format, player::Participants};

// FIXME remove since all formats will have their own structs+methods
/// Initialize a new bracket with participants and format (but not matches)
#[allow(dead_code)]
#[derive(Default)]
pub struct Builder {
    /// Format of bracket
    format: Option<Format>,
    /// Participants of bracket
    participants: Option<Participants>,
}

/// Error while building bracket
#[derive(Debug)]
pub enum Error {
    /// Missing format
    MissingFormat,
    /// Missing players
    MissingPlayers,
}
