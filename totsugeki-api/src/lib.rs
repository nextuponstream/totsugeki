#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]
#![doc = include_str!("../README.md")]

pub mod bracket;
pub mod join;
pub mod persistence;
pub mod routes;

use crate::persistence::{inmemory::InMemoryDBAccessor, DBAccessor, Error};
use bracket::BracketGETResponse;
use hmac::{Hmac, NewMac};
use jwt::VerifyWithKey;
use log::{error, warn};
use persistence::postgresql::PostgresqlDBAccessor;
use poem::{http::Method, middleware::Cors, web::Data, EndpointExt, Request, Route};
use poem_openapi::{auth::ApiKey, Object, OpenApiService, SecurityScheme};
use routes::bracket::BracketApi;
use routes::join::JoinApi;
use routes::organiser::OrganiserApi;
use routes::service::ServiceApi;
use routes::test_utils::TestUtilsApi;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::boxed::Box;
use std::collections::HashMap;
use std::sync::Arc;
use totsugeki::bracket::{ActiveBrackets, BracketId};
use totsugeki::organiser::{Organiser, OrganiserId};
use totsugeki::ReadLock;
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
pub fn hmac(server_key: &[u8]) -> Hmac<Sha256> {
    Hmac::<Sha256>::new_from_slice(server_key).expect("valid server key")
}

#[derive(Serialize, Deserialize, Object)]
/// Organiser POST body response
pub struct OrganiserPOSTResponse {
    organiser_name: String,
}

fn log_error(e: &Error) {
    match e {
        Error::PoisonedReadLock(e) => error!("{e}"),
        Error::PoisonedWriteLock(e) => error!("{e}"),
        Error::Code(e) | Error::Unknown(e) => error!("{e}"),
        Error::Denied(e) => warn!("{e}"),
        Error::Parsing(e) => warn!("User input could not be parsed: {e}"),
    }
}

/// Type of supported services
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
pub type FinalizedBrackets = HashMap<BracketId, BracketGETResponse>;

#[derive(Object, Serialize, Deserialize)]
/// Organiser GET response
pub struct OrganiserGETResponse {
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

impl From<Organiser> for OrganiserGETResponse {
    fn from(o: Organiser) -> Self {
        Self {
            organiser_id: o.get_organiser_id(),
            organiser_name: o.get_organiser_name(),
            active_brackets: o.get_active_brackets(),
            finalized_brackets: {
                let mut map = HashMap::new();
                for kv in o.get_finalized_brackets().iter() {
                    map.insert(*kv.0, BracketGETResponse::new(kv.1.clone()));
                }
                map
            },
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

async fn api_checker(req: &Request, api_key: ApiKey) -> Option<ApiServiceUser> {
    let server_key = req.data::<ServerKey>().unwrap();
    VerifyWithKey::<ApiServiceUser>::verify_with_key(api_key.key.as_str(), server_key).ok()
}

/// OAI service for tests
pub fn oai_test_service(
) -> OpenApiService<(BracketApi, OrganiserApi, ServiceApi, JoinApi, TestUtilsApi), ()> {
    OpenApiService::new(
        (BracketApi, OrganiserApi, ServiceApi, JoinApi, TestUtilsApi),
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

type TotsugekiEndpoint = poem::middleware::AddDataEndpoint<
    poem::middleware::AddDataEndpoint<
        poem::middleware::CorsEndpoint<Route>,
        Arc<ReadLock<Database>>,
    >,
    Hmac<Sha256>,
>;

/// Return route with cors enabled, authorization and database
pub fn route_with_data(
    route: Route,
    db_type: DatabaseType,
    server_key: Vec<u8>,
) -> TotsugekiEndpoint {
    let db: Database = db_type.into();
    let db = Arc::new(ReadLock::new(db));
    db.read()
        .expect("database")
        .init()
        .expect("initialise database");
    let server_key = hmac(&server_key);
    let cors = Cors::new().allow_method(Method::GET);
    // NOTE: use lsp "add return type" after deleting fn return type instead of
    // guessing
    route.with(cors).data(db).data(server_key)
}
