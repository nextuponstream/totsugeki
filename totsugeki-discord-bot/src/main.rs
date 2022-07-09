use serenity::async_trait;
use serenity::framework::standard::macros::group;
use serenity::framework::standard::StandardFramework;
use serenity::prelude::*;
use std::env;
use std::sync::Arc;
use totsugeki_discord_bot::commands::bracket::*;
use totsugeki_discord_bot::commands::help::*;
use totsugeki_discord_bot::commands::ping::*;
use totsugeki_discord_bot::TournamentServer;

#[group]
#[commands(ping)]
#[summary = "Main available commands"]
#[sub_groups("bracket")]
pub struct General;
struct Handler;

#[async_trait]
impl EventHandler for Handler {}

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("error"));

    dotenv::dotenv().expect("Failed to load .env file");

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("!")) // bot prefix
        .help(&HELP)
        .group(&GENERAL_GROUP);

    // Login with a bot token from the environment
    let token_path = env::var("DISCORD_BOT_TOKEN_PATH").expect("DISCORD_BOT_TOKEN_PATH");
    let token = std::fs::read_to_string(token_path).expect("discord bot token secret");
    let intents = GatewayIntents::non_privileged() | GatewayIntents::MESSAGE_CONTENT;
    let mut client = Client::builder(token, intents)
        .event_handler(Handler)
        .framework(framework)
        .await
        .expect("Error creating client");

    {
        let mut data = client.data.write().await;
        let addr = env::var("API_ADDR").expect("API_ADDR");
        let port = env::var("API_PORT").ok();
        let accept_invalid_certs =
            env::var("ACCEPT_INVALID_CERTS_FROM_API").expect("ACCEPT_INVALID_CERTS_FROM_API");
        let accept_invalid_certs = accept_invalid_certs
            .parse::<bool>()
            .expect("could not parse ACCEPT_INVALID_CERTS_FROM_API");
        let api_token_path =
            env::var("API_TOKEN_FOR_DISCORD_BOT_PATH").expect("API_TOKEN_FOR_DISCORD_BOT_PATH");
        let api_token = std::fs::read_to_string(api_token_path).expect("api token secret");
        let server = TournamentServer::new(addr, port, accept_invalid_certs, api_token);
        data.insert::<TournamentServer>(Arc::new(server));
    }

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
