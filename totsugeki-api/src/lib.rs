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

use crate::persistence::DBAccessor;
use hmac::{Hmac, NewMac};
use jwt::VerifyWithKey;
use poem::web::Data;
use poem::Request;
use poem_openapi::{auth::ApiKey, Object, SecurityScheme};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::boxed::Box;
use std::sync::Arc;
use totsugeki::Bracket;
use totsugeki::ReadLock;

/// Server key encryption type
pub type ServerKey = Hmac<Sha256>;

/// Instance of shared database
// NOTE: There is no read lock in standard lib. Since it's only a handle, we never write to the underlying
// struct. Then any number of reader can use it since no writer will prevent readers from using methods
// tied to the struct to interact with the database.
pub type SharedDb<'a> = Data<&'a Arc<ReadLock<Box<dyn DBAccessor + Send + Sync>>>>;

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

#[derive(Debug, Serialize, Deserialize)]
/// Discord bot api user
pub struct DiscordApiUser {}

#[derive(SecurityScheme)]
#[oai(
    type = "api_key",
    key_name = "X-API-Key",
    in = "header",
    checker = "api_checker"
)]
struct MyApiKeyAuthorization(DiscordApiUser);

async fn api_checker(req: &Request, api_key: ApiKey) -> Option<DiscordApiUser> {
    let server_key = req.data::<ServerKey>().unwrap();
    VerifyWithKey::<DiscordApiUser>::verify_with_key(api_key.key.as_str(), server_key).ok()
}

/// Returns HMAC from server key
pub fn hmac(server_key: &[u8]) -> Hmac<Sha256> {
    Hmac::<Sha256>::new_from_slice(server_key).expect("valid server key")
}

#[derive(Serialize, Deserialize, Object)]
/// Organiser (venue, organisation) for a tournament
pub struct OrganiserPOST {
    organiser_name: String,
}
