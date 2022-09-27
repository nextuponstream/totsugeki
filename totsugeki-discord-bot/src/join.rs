//! Join bracket

use crate::{Config, Data};
use std::{io::prelude::*, path::Path};
// use async_fs::File;
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki::player::Player;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Join bracket"]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "Join bracket command");
    span.in_scope(|| async {
        let name = msg.author.name.clone();
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (mut bracket, mut users) = bracket_data.clone();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access

        let player = match users.get(&user_id) {
            Some(p) => p.clone(),
            None => Player::new(name),
        };
        users.insert(user_id, player.clone());

        match bracket.clone().add_new_player(player.clone()) {
            Ok(b) => {
                bracket = b;
            }
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        *bracket_data = (bracket, users);

        let j = serde_json::to_string(&bracket_data.clone()).expect("bracket");
        f.write_all(j.as_bytes())?;
        msg.reply(ctx, format!("You joined as {player}")).await?;
        info!("{player} joined");
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
