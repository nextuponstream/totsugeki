#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

pub mod persistence;
pub mod routes;

use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use totsugeki::Bracket;

#[derive(Serialize, Deserialize, Object)]
/// Bracket for a tournament
pub struct BracketPOST {
    bracket_name: String,
}

/// Bracket for a tournament
//
// NOTE: having Bracket implement `ToJSON` means that importing `totsugeki` will bring in all poem
// dependencies. This does not play nice with yew dependencies when doing relative import
// (totsugeki = { path = "../totsugeki" }) and caused many errors. The workaround is to leave
// Bracket package as barebones as possible and let packages importing it the task of deriving
// necessary traits into their own structs.
#[derive(Object)]
pub struct BracketGET {
    id: i64,
    bracket_name: String,
}

impl BracketGET {
    /// Form values to be sent to the API to create a bracket
    #[must_use]
    pub fn new(bracket: Bracket) -> Self {
        BracketGET {
            id: bracket.get_id(),
            bracket_name: bracket.get_bracket_name(),
        }
    }
}

impl From<Bracket> for BracketGET {
    fn from(b: Bracket) -> Self {
        BracketGET::new(b)
    }
}
