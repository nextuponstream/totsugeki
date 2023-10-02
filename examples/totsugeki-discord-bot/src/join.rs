//! Join bracket

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
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

        let player = match users.get(&user_id) {
            Some(p) => p.clone(),
            None => Player::new(name),
        };
        users.insert(user_id, player.clone());

        match bracket.clone().join(player.clone()) {
            Ok(b) => {
                bracket = b;
            }
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        *bracket_data = (bracket.clone(), users.clone());

        let d = Data {
            bracket: bracket.clone(),
            users: users.clone(),
        };
        let j = serde_json::to_string(&d).expect("bracket");

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has less chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("{player} joined");
        msg.reply(ctx, format!("You joined as {player}")).await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
