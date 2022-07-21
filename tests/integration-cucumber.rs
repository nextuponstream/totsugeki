//! Test requests to backend by external services (for instance, frontend or discord bot) using request library.
//! Theses tests should be either ran sequentially or setup to spawn multiple databases
//! instances to use, then ran in parallel.

// TODO create test setup to run in parralel

#![deny(missing_docs)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

use async_trait::async_trait;
use cucumber::WorldInit;
use futures::FutureExt as _;
use std::convert::Infallible;
use std::env;
use totsugeki::bracket::BracketId;
use totsugeki::organiser::OrganiserId;
use totsugeki_api_request::{clean_database, get_service_token, RequestError};
use totsugeki_discord_bot::DiscordChannel;

// NOTE: no ü or any fancy caracters

mod cucumber_expressions;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    World::cucumber()
        .fail_on_skipped()
        .before(move |_feature, _rule, _scenario, world| {
            async move {
                world.clean().await.expect("clean world");
                clean_database(
                    get_client(world.accept_invalid_certs),
                    world.tournament_server_addr.as_str(),
                    world
                        .authorization_header
                        .clone()
                        .expect("authorization header")
                        .as_str(),
                )
                .await
                .expect("could not clean database before executing test")
            }
            .boxed()
        })
        .run("tests/features/")
        .await;
}

#[async_trait(?Send)]
impl cucumber::World for World {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        dotenv::dotenv().expect("Failed to load .env file");
        let addr = env::var("API_ADDR").expect("API_ADDR");
        let port = env::var("API_PORT").expect("API_PORT");
        let accept_invalid_certs = env::var("ACCEPT_INVALID_CERTS_FROM_API")
            .expect("ACCEPT_INVALID_CERTS_FROM_API")
            .parse::<bool>()
            .expect("could not parse ACCEPT_INVALID_CERTS_FROM_API");
        Ok(Self {
            tournament_server_addr: format!("{addr}:{port}"),
            user: None,
            bracket_name: None,
            accept_invalid_certs,
            authorization_header: None,
            organiser_name: None,
            bracket_id: None,
            organiser_id: None,
            organiser_internal_id: None,
            discussion_channel_id: None,
        })
    }
}

#[derive(Debug, WorldInit)]
/// Api under test
pub struct World {
    tournament_server_addr: String,
    accept_invalid_certs: bool,
    authorization_header: Option<String>,
    user: Option<String>,
    bracket_name: Option<String>,
    organiser_name: Option<String>,
    bracket_id: Option<BracketId>,
    organiser_id: Option<OrganiserId>,
    organiser_internal_id: Option<String>,
    discussion_channel_id: Option<DiscordChannel>,
}

impl World {
    async fn clean(&mut self) -> Result<(), RequestError> {
        self.user = None;
        self.bracket_name = None;
        self.authorization_header = None;
        self.organiser_name = None;
        self.bracket_id = None;
        self.organiser_id = None;
        self.organiser_internal_id = None;
        self.discussion_channel_id = None;

        let res = get_service_token(get_client(true), self.tournament_server_addr.as_str()).await?;
        self.authorization_header = Some(res.get_token());
        Ok(())
    }
}

/// Return http client
pub fn get_client(accept_invalid_certs: bool) -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .expect("http client")
}
