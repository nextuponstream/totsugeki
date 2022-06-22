#![warn(clippy::pedantic)]

use async_trait::async_trait;
use cucumber::{given, then, when, WorldInit};
use futures::FutureExt as _;
use std::convert::Infallible;
use std::env;
use std::time::Duration;
use tokio::time::sleep;
use totsugeki::discord_commands::bracket::create::bot_create_bracket;
use totsugeki::discord_commands::bracket::find::bot_find_bracket;
use totsugeki::discord_commands::bracket::get::bot_get_bracket;

// NOTE: no ü or any fancy caracters

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    World::cucumber()
        .fail_on_skipped()
        .before(|_feature, _rule, _scenario, world| {
            world.clean();
            sleep(Duration::from_millis(10)).boxed_local()
        })
        .run("tests/features/")
        .await;
}

#[async_trait(?Send)]
impl cucumber::World for World {
    type Error = Infallible;

    async fn new() -> Result<Self, Self::Error> {
        dotenv::from_filename(".env-test").expect("Failed to load .env-test file");
        let addr = env::var("TOURNAMENT_SERVER_ADDR").expect("TOURNAMENT_SERVER_ADDR");
        let port = env::var("TOURNAMENT_SERVER_PORT").expect("TOURNAMENT_SERVER_PORT");
        let accept_invalid_certs = env::var("ACCEPT_INVALID_CERTS").expect("ACCEPT_INVALID_CERTS");
        let accept_invalid_certs = accept_invalid_certs
            .parse::<bool>()
            .expect("could not parse ACCEPT_INVALID_CERTS");
        Ok(Self {
            tournament_server_addr: format!("{addr}:{port}"),
            user: None,
            bracket_name: None,
            accept_invalid_certs,
        })
    }
}

#[derive(Debug, WorldInit)]
struct World {
    tournament_server_addr: String,
    user: Option<String>,
    bracket_name: Option<String>,
    accept_invalid_certs: bool,
}

impl World {
    fn clean(&mut self) {
        self.user = None;
        self.bracket_name = None;
    }
}

#[given(expr = "{word} wants to create(s|) a bracket named {word}")] // Cucumber Expression
fn someone_wants_to_create_bracket(w: &mut World, user: String, bracket_name: String) {
    w.user = Some(user);
    w.bracket_name = Some(bracket_name);
}

#[when(regex = r"^(?:he|she|they) create(s|) a bracket using discord bot")]
async fn create_bracket(w: &mut World) {
    if let Err(e) = bot_create_bracket(
        w.tournament_server_addr.clone(),
        w.bracket_name.clone().unwrap(),
        w.accept_invalid_certs,
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
    let brackets =
        bot_get_bracket(w.tournament_server_addr.clone(), 0, w.accept_invalid_certs).await;
    let brackets = brackets.unwrap();
    let bracket_name = w.bracket_name.clone().unwrap();
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
    let bracket_name = w.bracket_name.clone().unwrap();
    let brackets = bot_find_bracket(
        w.tournament_server_addr.clone(),
        bracket_name.clone(),
        0,
        w.accept_invalid_certs,
    )
    .await;
    let brackets = brackets.unwrap();
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
