//! Seed bracket command
//! Join bracket

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki::player::Id as PlayerId;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Seed bracket by providing an ordered list of player IDs"]
#[usage = "<PLAYER IDS...>"]
#[allowed_roles("TO")]
async fn seed(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let span = span!(Level::INFO, "Seed bracket command");
    span.in_scope(|| async {
        let players = args.iter::<PlayerId>().collect::<Result<Vec<_>, _>>()?;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (mut bracket, users) = bracket_data.clone();

        match bracket.clone().update_seeding(&players) {
            Ok(b) => {
                bracket = b;
            }
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        let players = bracket.get_participants().get_players_list();
        let mut new_seeding_message = "".to_string();
        for p in players {
            new_seeding_message = format!("{new_seeding_message}\n- {p}");
        }
        *bracket_data = (bracket.clone(), users.clone());

        let d = Data {
            bracket: bracket.clone(),
            users: users.clone(),
        };
        let j = serde_json::to_string(&d).expect("bracket");

        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has less chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("Seeding updated");
        msg.reply(ctx, format!("Seeding updated: {new_seeding_message}"))
            .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
