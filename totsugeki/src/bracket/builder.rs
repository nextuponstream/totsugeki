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

impl Builder {
    /// Build bracket
    ///
    /// # Errors
    /// Throws error when a required attribute is missing (example: format)
    /// # Panics
    /// when math overflow happens
    pub fn build(self) -> Result<Bracket, Error> {
        let Some(format) = self.format else {
            return Err(Error::MissingFormat);
        };
        let Some(participants) = self.participants else {
            return Err(Error::MissingPlayers);
        };

        let mut bracket = Bracket {
            format,
            ..Bracket::default()
        };
        bracket = bracket
            .regenerate_matches(participants)
            .expect("no math overflow errors");

        Ok(bracket)
    }

    /// Set format for bracket to build
    #[must_use]
    pub fn set_format(self, format: Format) -> Builder {
        Builder {
            format: Some(format),
            ..self
        }
    }

    /// Set `n` participants of bracket to build
    ///
    /// # Panics
    /// We add new players sequentially and generate new UUID v4 everytime.
    /// We do not expect two same UUID to be generated and cause a panic for
    /// adding two same players.
    #[must_use]
    pub fn set_new_players(self, n: usize) -> Builder {
        use crate::player::Player;

        let mut participants = Participants::default();
        for i in 1..=n {
            participants = participants
                .add_participant(Player::new(format!("p{i}")))
                .expect("players");
        }
        Builder {
            participants: Some(participants),
            ..self
        }
    }
}
