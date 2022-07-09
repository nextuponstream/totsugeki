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
use cucumber::{given, then, when, WorldInit};
use futures::FutureExt as _;
use std::convert::Infallible;
use std::env;
use totsugeki_api_request::bracket::create;
use totsugeki_api_request::bracket::fetch;
use totsugeki_api_request::clean_database;

// NOTE: no ü or any fancy caracters

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
        })
    }
}

#[derive(Debug, WorldInit)]
struct World {
    tournament_server_addr: String,
    user: Option<String>,
    bracket_name: Option<String>,
    accept_invalid_certs: bool,
    authorization_header: String,
}

impl World {
    fn clean(&mut self) {
        self.user = None;
        self.bracket_name = None;
    }
}

#[given(expr = "{word} wants to create(s|) a bracket named {word}")]
fn someone_wants_to_create_bracket(w: &mut World, user: String, bracket_name: String) {
    w.user = Some(user);
    w.bracket_name = Some(bracket_name);
}

#[when(regex = r"^(?:he|she|they) create(s|) a bracket using discord bot")]
async fn create_bracket(w: &mut World) {
    if let Err(e) = create(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        w.authorization_header.as_str(),
        w.bracket_name
            .clone()
            .expect("no bracket name provided")
            .as_str(),
    )
    .await
    {
        panic!("bracket could not be created: {e}");
    }
}

#[then(
    regex = r"^(?:he|she|they) search the newly created bracket with the discord bot and find it"
)]
async fn see_bracket(w: &mut World) {
    let brackets = fetch(get_client(true), w.tournament_server_addr.as_str(), None, 0)
        .await
        .expect("could not fetch brackets");
    let bracket_name = w.bracket_name.clone().expect("no bracket name");
    assert!(
        brackets
            .into_iter()
            .any(|b| b.get_bracket_name() == bracket_name),
        "did not find \"{}\"",
        bracket_name
    );
}

#[then(regex = r"^(?:he|she|they) can filter results and find the created bracket")]
async fn find_bracket(w: &mut World) {
    let bracket_name = w.bracket_name.clone().expect("no bracket name");
    let brackets = fetch(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        Some(bracket_name.clone()),
        0,
    )
    .await;
    let brackets = brackets.expect("no brackets received");
    assert!(
        brackets
            .clone()
            .into_iter()
            .any(|b| b.get_bracket_name() == bracket_name),
        "did not find filtered bracker \"{}\"",
        bracket_name
    );
    assert!(brackets.len() == 1, "too many results");
}

fn get_client(accept_invalid_certs: bool) -> reqwest::Client {
    reqwest::Client::builder()
        .danger_accept_invalid_certs(accept_invalid_certs)
        .build()
        .expect("http client")
}
