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

use crate::persistence::{DBAccessor, Error};
use hmac::{Hmac, NewMac};
use jwt::VerifyWithKey;
use log::{error, warn};
use poem::web::Data;
use poem::Request;
use poem_openapi::{auth::ApiKey, Object, SecurityScheme};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::Arc;
use totsugeki::bracket::ActiveBrackets;
use totsugeki::bracket::{Bracket, BracketId};
use totsugeki::organiser::Organiser;
use totsugeki::DiscussionChannelId;
use totsugeki::OrganiserId;
use totsugeki::ReadLock;
use uuid::Uuid;

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
    organiser_name: String,
    channel_internal_id: String,
    organiser_internal_id: String,
    service_type_id: String,
}

/// Bracket for a tournament
//
// NOTE: having Bracket implement `ToJSON` means that importing `totsugeki` will bring in all poem
// dependencies. This does not play nice with yew dependencies when doing relative import
// (totsugeki = { path = "../totsugeki" }) and caused many errors. The workaround is to leave
// Bracket package as barebones as possible and let packages importing it the task of deriving
// necessary traits into their own structs.
#[derive(Object, Serialize, Deserialize)]
pub struct BracketGET {
    id: BracketId,
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

/// Returns HMAC from server key
pub fn hmac(server_key: &[u8]) -> Hmac<Sha256> {
    Hmac::<Sha256>::new_from_slice(server_key).expect("valid server key")
}

#[derive(Serialize, Deserialize, Object)]
/// Organiser (venue, organisation) for a tournament
pub struct OrganiserPOST {
    organiser_name: String,
}

fn log_error(e: &Error) {
    match e {
        Error::PoisonedReadLock(e) => error!("{e}"),
        Error::PoisonedWriteLock(e) => error!("{e}"),
        Error::Code(e) | Error::Unknown(e) => error!("{e}"),
        Error::Denied() => warn!("Unauthorized action from user"),
        Error::Parsing(e) => warn!("User input could not be parsed: {e}"),
    }
}

/// Type of services supported
pub enum InternalIdType {
    /// Discord
    Discord,
}

impl std::fmt::Display for InternalIdType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InternalIdType::Discord => write!(f, "discord"),
        }
    }
}

/// Finalized brackets
pub type FinalizedBrackets = HashMap<BracketId, BracketGET>;

#[derive(Object, Serialize, Deserialize)]
/// Get organisers
pub struct OrganiserGET {
    organiser_id: OrganiserId,
    organiser_name: String,
    active_brackets: ActiveBrackets,
    finalized_brackets: FinalizedBrackets,
}

/// API service identifier
type ApiServiceId = Uuid;

/// Api service
#[derive(Object, Serialize, Deserialize)]
pub struct ApiServiceUser {
    id: ApiServiceId,
    name: String,
    description: String,
}

impl ApiServiceUser {
    #[must_use]
    /// Create new Api service (like a discord bot)
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: ApiServiceId::new_v4(),
            name,
            description,
        }
    }
}

#[derive(Object)]
/// Response body
pub struct BracketPOSTResult {
    bracket_id: BracketId,
    organiser_id: OrganiserId,
    discussion_channel_id: DiscussionChannelId,
}

impl BracketPOSTResult {
    #[must_use]
    /// Create response body
    pub fn new(
        bracket_id: BracketId,
        organiser_id: OrganiserId,
        discussion_channel_id: DiscussionChannelId,
    ) -> Self {
        Self {
            bracket_id,
            organiser_id,
            discussion_channel_id,
        }
    }
}

impl From<Organiser> for OrganiserGET {
    fn from(o: Organiser) -> Self {
        Self {
            organiser_id: o.get_organiser_id(),
            organiser_name: o.get_organiser_name(),
            active_brackets: o.get_active_brackets(),
            finalized_brackets: {
                let mut map = HashMap::new();
                for kv in o.get_finalized_brackets().iter() {
                    map.insert(*kv.0, BracketGET::new(kv.1.clone()));
                }
                map
            },
        }
    }
}

/// Api key service authorization
#[derive(SecurityScheme)]
#[oai(
    type = "api_key",
    key_name = "X-API-Key",
    in = "header",
    checker = "api_checker"
)]
pub struct ApiKeyServiceAuthorization(ApiServiceUser);

async fn api_checker(req: &Request, api_key: ApiKey) -> Option<ApiServiceUser> {
    let server_key = req.data::<ServerKey>().unwrap();
    VerifyWithKey::<ApiServiceUser>::verify_with_key(api_key.key.as_str(), server_key).ok()
}
