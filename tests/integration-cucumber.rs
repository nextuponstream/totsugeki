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
use totsugeki_api_request::clean_database;

// NOTE: no ü or any fancy caracters

mod cucumber_expressions;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    World::cucumber()
        .fail_on_skipped()
        .before(move |_feature, _rule, _scenario, world| {
            async move {
                world.clean();
                clean_database(
                    get_client(world.accept_invalid_certs),
                    world.tournament_server_addr.as_str(),
                    world.authorization_header.as_str(),
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
        dotenv::from_filename(".env-test").expect("Failed to load .env-test file");
        let addr = env::var("API_ADDR").expect("API_ADDR");
        let port = env::var("API_PORT").expect("API_PORT");
        let accept_invalid_certs =
            env::var("ACCEPT_INVALID_CERTS_FROM_API").expect("ACCEPT_INVALID_CERTS_FROM_API");
        let accept_invalid_certs = accept_invalid_certs
            .parse::<bool>()
            .expect("could not parse ACCEPT_INVALID_CERTS_FROM_API");
        let discord_api_token_path =
            env::var("API_TOKEN_FOR_DISCORD_BOT_PATH").expect("API_TOKEN_FOR_DISCORD_BOT_PATH");
        let authorization_header =
            std::fs::read(discord_api_token_path).expect("discord api token secret");
        let authorization_header: &str =
            std::str::from_utf8(&authorization_header).expect("server_key file content");
        Ok(Self {
            tournament_server_addr: format!("{addr}:{port}"),
            user: None,
            bracket_name: None,
            accept_invalid_certs,
            authorization_header: authorization_header.to_string(),
            organiser_name: None,
        })
    }
}

#[derive(Debug, WorldInit)]
/// Api under test
pub struct World {
    tournament_server_addr: String,
    accept_invalid_certs: bool,
    authorization_header: String,
    user: Option<String>,
    bracket_name: Option<String>,
    organiser_name: Option<String>,
}

impl World {
    fn clean(&mut self) {
        self.user = None;
        self.bracket_name = None;
        self.organiser_name = None;
    }
}

/// Return http client
pub fn get_client(accept_invalid_certs: bool) -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .expect("http client")
}
