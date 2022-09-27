use async_lock::RwLock;
use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use std::io::Write;
use std::path::Path;
use std::sync::Arc;
use std::{collections::HashMap, env};
use totsugeki::bracket::Bracket;
use totsugeki_discord_bot::{create::*, join::*, ping::*, Config, Data};
use tracing::subscriber::set_global_default;
use tracing_bunyan_formatter::BunyanFormattingLayer;
use tracing_log::LogTracer;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

// static BRACKET: RwLock<Bracket> = RwLock::new(Bracket::default());

// if you want to use Data, you need a dyn trait which implements boxed and
// sync.
// Or you can read from file everytime.

#[group]
#[commands(ping, create, join)]
#[summary = "Main available commands"]
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
        // .help(&HELP)
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
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(p)
            .expect("");
        let bracket_data = std::fs::read(p).expect("file");
        let bracket_data = match serde_json::from_slice::<Data>(&bracket_data) {
            Ok(d) => d,
            Err(_) => Data {
                bracket: Bracket::default(),
                users: HashMap::new(),
            },
        };
        data.insert::<Data>(Arc::new(RwLock::new((
            bracket_data.bracket.clone(),
            bracket_data.users.clone(),
        ))));
        let j = serde_json::to_string(&bracket_data).expect("bracket");
        f.write_all(j.as_bytes()).expect("write");
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
