use async_lock::RwLock;
use serenity::async_trait;
use serenity::client::{Client, EventHandler};
use serenity::framework::standard::macros::group;
use serenity::framework::StandardFramework;
use serenity::prelude::*;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::{collections::HashMap, env};
use totsugeki::bracket::Bracket;
use totsugeki_discord_bot::{
    close::*, create::*, disqualify::*, forfeit::*, help::*, join::*, next_match::*, ping::*,
    players::*, quit::*, remove::*, report::*, seed::*, start::*, validate::*, Config, Data,
};
use tracing::subscriber::set_global_default;
use tracing::warn;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

#[group]
#[commands(
    ping,
    create,
    join,
    next_match,
    report,
    start,
    tournament_organiser_reports,
    validate,
    quit,
    players,
    seed,
    remove,
    disqualify,
    forfeit,
    close
)]
#[summary = "Main available commands"]
#[description = "Manage bracket with this command"]
pub struct General;
struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    LogTracer::init().expect("Failed to set logger");
    let formatting_layer = BunyanFormattingLayer::new(
        "totsugeki-discord-bot".into(),
        // Output the formatted spans to stdout.
        std::io::stdout,
    );
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let subscriber = Registry::default().with(env_filter).with(formatting_layer);
    set_global_default(subscriber).expect("Failed to set subscriber.");

    dotenv::dotenv().expect("Failed to load .env file");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // bot prefix
        .help(&HELP)
        .group(&GENERAL_GROUP);

    let bracket_filename =
        env::var("BRACKET_FILENAME").expect("BRACKET_FILENAME environment variable");

    // Login with a bot token from the environment
    let token_path = env::var("DISCORD_BOT_TOKEN_PATH").expect("DISCORD_BOT_TOKEN_PATH");
    let token = std::fs::read_to_string(token_path).expect("discord agent token secret");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<Config>(Arc::new(bracket_filename.clone()));
        let p = Path::new(&bracket_filename);
        let bracket_data = match std::fs::read(p) {
            Ok(r) => r,
            Err(e) => {
                warn!("could not parse read file: {e}");
                vec![]
            }
        };
        let bracket_data = match serde_json::from_slice::<Data>(&bracket_data) {
            Ok(d) => d,
            Err(e) => {
                warn!("could not parse file: {e}");
                Data {
                    bracket: Bracket::default(),
                    users: HashMap::new(),
                }
            }
        };
        data.insert::<Data>(Arc::new(RwLock::new((
            bracket_data.bracket.clone(),
            bracket_data.users.clone(),
        ))));
        let j = serde_json::to_vec(&bracket_data).expect("bracket");
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(p)
            .expect("");
        f.write_all(&j).expect("write");
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
