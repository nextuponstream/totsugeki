//! dioxus UI components and functions to display a bracket

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../README.md")]
#![warn(clippy::unwrap_used)]

use crate::components::bracket::displayable_match::EMPTY_NAME;
use totsugeki::matches::Id as MatchId;

pub mod components;

/// Maximum size for name
const MAX_NAME_SIZE: usize = 64;
/// Name that can be copied over
type Name = [u8; MAX_NAME_SIZE];

/// A struct to represent names in UI that contains maximum `MAX_NAME_SIZE`
/// caracters
struct ShortName {
    /// Name as bytes
    pub(crate) value: [u8; MAX_NAME_SIZE],
}

impl ShortName {
    /// Get utf8 string for name
    pub(crate) fn get(&self) -> String {
        // TODO remove expect
        String::from_utf8(self.value.into()).expect("string")
    }
}

impl std::default::Default for ShortName {
    fn default() -> Self {
        let mut value = String::default()
            .into_bytes()
            .into_iter()
            .collect::<Vec<u8>>();
        value.resize(MAX_NAME_SIZE, 0);
        // let value = value.try_into().unwrap();
        let value = EMPTY_NAME;
        Self { value }
    }
}

/// Converts name of player of totsugeki library into displayable name of fixed
/// size `MAX_NAME_SIZE` to pass around as argument
pub(crate) fn convert_to_displayable_name(name: String) -> Name {
    let mut name = name
        .into_bytes()
        .into_iter()
        .take(MAX_NAME_SIZE)
        .collect::<Vec<u8>>();
    name.resize(MAX_NAME_SIZE, 0); // '\0' null byte
    match name.try_into() {
        Ok(n) => n,
        Err(_e) => {
            // TODO log error
            EMPTY_NAME
        }
    }
}

/// All types of modals necessary for updating a bracket
pub enum Modal {
    /// Add player to bracket
    AddPlayer,
    /// Enter result for given `MatchId` between player 1 and player 2
    EnterMatchResult(MatchId, Name, Name),
    /// Disqualify player from bracket
    Disqualify,
}

#[cfg(test)]
mod test {
    use crate::ShortName;

    #[test]
    fn get_default_short_name() {
        ShortName::default();
    }
}
