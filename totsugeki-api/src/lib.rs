#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

pub mod persistence;
pub mod routes;

use crate::persistence::{inmemory::InMemoryDBAccessor, DBAccessor, Error};
use hmac::{Hmac, NewMac};
use jwt::VerifyWithKey;
use persistence::postgresql::Accessor as PostgresqlDBAccessor;
use poem::{http::Method, middleware::Cors, web::Data, EndpointExt, Request, Route};
use poem_openapi::{auth::ApiKey, Object, OpenApiService, SecurityScheme};
use routes::{
    bracket::Api as BracketApi, health_check::Api as HealthcheckApi, join::Api as JoinApi,
    organiser::Api as OrganiserApi, service::Api as ServiceApi, test_utils::Api as TestUtilsApi,
};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::boxed::Box;
use std::collections::HashSet;
use std::sync::Arc;
use totsugeki::{
    bracket::{Id as BracketId, GET},
    ReadLock,
};
use tracing::{error, warn};
use uuid::Uuid;

/// Database accessor
pub type Database = Box<dyn DBAccessor + Send + Sync>;

/// Server key encryption type
pub type ServerKey = Hmac<Sha256>;

/// Instance of shared database
// NOTE: There is no read lock in standard lib. Since it's only a handle, we never write to the underlying
// struct. Then any number of reader can use it since no writer will prevent readers from using methods
// tied to the struct to interact with the database.
pub type SharedDb<'a> = Data<&'a Arc<ReadLock<Box<dyn DBAccessor + Send + Sync>>>>;

/// Returns HMAC from server key
#[must_use]
pub fn hmac(server_key: &[u8]) -> Hmac<Sha256> {
    Hmac::<Sha256>::new_from_slice(server_key).expect("valid server key")
}

/// Log error are the appropriate level
fn log_error(e: &Error) {
    match e {
        Error::PoisonedReadLock(e) => error!("{e}"),
        Error::PoisonedWriteLock(e) => error!("{e}"),
        Error::Code(e) | Error::Unknown(e) => error!("{e}"),
        Error::Denied(e) => warn!("{e}"),
        Error::Parsing(e) => warn!("User input could not be parsed: {e}"),
        Error::UnregisteredBracket(b_id) => warn!("User searched for unknown bracket: {b_id}"),
        Error::UnregisteredDiscussionChannel(service, id) => {
            warn!("Unregistered discussion channel requested for service \"{service}\" with id \"{id}\"");
        }
        Error::NoActiveBracketInDiscussionChannel => {
            warn!("User did not find active bracket in discussion channel");
        }
        Error::PlayerNotFound => warn!("A user searched for an unknown player"),
        Error::NextMatchNotFound => error!("User could not get their next match"),
        // TODO add more info for debugging
        Error::NoNextMatch => {
            warn!("User wanted to know their next match but there is none for them");
        }
        Error::Seeding(e) => error!("Seeding is impossible: {e}"),
        Error::NoOpponent => warn!("User searched for opponent but there was none"),
        Error::Match(e) => warn!("User could not update match status: {e}"),
        Error::EliminatedFromBracket => {
            warn!("Player searched for their next match but they were eliminated from the bracket");
        }
        Error::OrganiserNotFound(service, id) => warn!("Requested organiser was not found. Organiser used service \"{service}\" with id: \"{id}\""),
        Error::BracketInactive(user_id, bracket_id) => warn!("User {user_id} reported a result for bracket \"{bracket_id}\" but it is inactive"),
        Error::UpdateBracket(e) => warn!("Bracket could not be updated: {e}"),
    }
}

/// Type of supported services
#[derive(Debug, Clone, Copy)]
pub enum Service {
    /// Discord
    Discord,
}

impl std::fmt::Display for Service {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Service::Discord => write!(f, "discord"),
        }
    }
}

/// Finalized brackets
pub type FinalizedBrackets = HashSet<BracketId>;

/// API service identifier
type ApiServiceId = Uuid;

/// Api service
#[derive(Object, Serialize, Deserialize)]
pub struct ApiServiceUser {
    /// Identifier of this service
    id: ApiServiceId,
    /// Name of this service
    name: String,
    /// A brief description of this service
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

/// Authorization mechanism for service to use endpoint
#[derive(SecurityScheme)]
#[oai(
    type = "api_key",
    key_name = "X-API-Key",
    in = "header",
    checker = "api_checker"
)]
pub struct ApiKeyServiceAuthorization(ApiServiceUser);

/// Authorization check for `api_key` holder
async fn api_checker(req: &Request, api_key: ApiKey) -> Option<ApiServiceUser> {
    let server_key = req.data::<ServerKey>().expect("server key");
    VerifyWithKey::<ApiServiceUser>::verify_with_key(api_key.key.as_str(), server_key).ok()
}

/// OAI service for tests
#[must_use]
pub fn oai_test_service() -> OpenApiService<
    (
        BracketApi,
        OrganiserApi,
        ServiceApi,
        JoinApi,
        HealthcheckApi,
        TestUtilsApi,
    ),
    (),
> {
    OpenApiService::new(
        (
            BracketApi,
            OrganiserApi,
            ServiceApi,
            JoinApi,
            HealthcheckApi,
            TestUtilsApi,
        ),
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
    )
}

/// Type of database in use by api
pub enum DatabaseType {
    /// In-memory database
    InMemory,
    /// Postgresql database
    Postgresql,
}

/// Error while inferring Database type from string
#[derive(Debug)]
pub enum ParseDatabaseTypeError {
    /// No match was found for string
    NoMatch,
}

impl std::str::FromStr for DatabaseType {
    type Err = ParseDatabaseTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "INMEMORY" => Ok(Self::InMemory),
            "POSTGRESQL" => Ok(Self::Postgresql),
            _ => Err(ParseDatabaseTypeError::NoMatch),
        }
    }
}

impl std::fmt::Display for DatabaseType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DatabaseType::InMemory => writeln!(f, "INMEMORY"),
            DatabaseType::Postgresql => writeln!(f, "POSTGRESQL"),
        }
    }
}

impl From<DatabaseType> for Database {
    fn from(db_type: DatabaseType) -> Self {
        match db_type {
            DatabaseType::InMemory => Box::new(InMemoryDBAccessor::default()),
            DatabaseType::Postgresql => Box::new(PostgresqlDBAccessor::default()),
        }
    }
}

/// Route type of Totsugeki api
type TotsugekiEndpoint = poem::middleware::AddDataEndpoint<
    poem::middleware::AddDataEndpoint<
        poem::middleware::CorsEndpoint<Route>,
        Arc<ReadLock<Database>>,
    >,
    Hmac<Sha256>,
>;

/// Return route with cors enabled, authorization and database
#[must_use]
pub fn route_with_data(
    route: Route,
    db_type: DatabaseType,
    server_key: &[u8],
) -> TotsugekiEndpoint {
    let db: Database = db_type.into();
    let db = Arc::new(ReadLock::new(db));
    db.read()
        .expect("database")
        .init()
        .expect("initialise database");
    let server_key = hmac(server_key);
    let cors = Cors::new().allow_method(Method::GET);
    // NOTE: use lsp "add return type" after deleting fn return type instead of
    // guessing
    route.with(cors).data(db).data(server_key)
}
