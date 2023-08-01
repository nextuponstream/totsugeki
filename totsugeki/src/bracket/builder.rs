//! Builer pattern for brackets

#[cfg(test)]
use crate::{bracket::Bracket, format::Format, player::Participants};

/// Initialize a new bracket with participants and format (but not matches)
#[cfg(test)]
pub struct Builder {
    /// Format of bracket
    format: Option<Format>,
    /// Participants of bracket
    participants: Option<Participants>,
}

/// Error while building bracket
#[cfg(test)]
#[derive(Debug)]
pub enum Error {
    /// Missing format
    MissingFormat,
    /// Missing players
    MissingPlayers,
}

#[cfg(test)]
impl Builder {
    /// Return new (invalid) builder
    #[cfg(test)]
    pub fn new() -> Builder {
        Builder {
            format: None,
            participants: None,
        }
    }

    /// Build bracket
    ///
    /// # Errors
    /// Throws error when a required attribute is missing (example: format)
    #[cfg(test)]
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
        bracket = bracket.regenerate_matches(participants).expect("ok");

        Ok(bracket)
    }

    /// Set format for bracket to build
    #[cfg(test)]
    pub fn set_format(self, format: Format) -> Builder {
        Builder {
            format: Some(format),
            ..self
        }
    }

    /// Set `n` participants of bracket to build
    #[cfg(test)]
    pub fn set_new_players(self, n: usize) -> Builder {
        use crate::player::Player;

        let mut participants = Participants::default();
        for i in 1..=n {
            participants = participants
                .add_participant(Player::new(format!("p{i}")))
                .expect("ok");
        }
        Builder {
            participants: Some(participants),
            ..self
        }
    }
}
