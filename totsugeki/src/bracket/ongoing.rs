//! Build `Bracket` instance from data

use super::{Bracket, Id};
use crate::{matches::Match, player::Participants};

impl Bracket {
    /// Build double-elimination bracket
    #[allow(dead_code)]
    #[must_use]
    pub fn assemble(
        id: Id,
        name: String,
        participants: Participants,
        matches: Vec<Match>,
    ) -> Bracket {
        Bracket {
            id,
            name,
            participants,
            matches,
            ..Bracket::default()
        }
    }
}
