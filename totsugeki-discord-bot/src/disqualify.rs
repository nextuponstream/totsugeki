//! Disqualify player from running bracket command

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
#[description = "Disqualify player from bracket"]
#[allowed_roles("TO")]
#[aliases("dq")]
async fn disqualify(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let span = span!(Level::INFO, "Disqualify player from bracket");
    span.in_scope(|| async {
        let player_id = args.single::<PlayerId>()?;
        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (mut bracket, users) = bracket_data.clone();

        match bracket.clone().disqualify_participant(player_id) {
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

        info!("{player_id} disqualified");
        msg.reply(ctx, format!("{player_id} was disqualified"))
            .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
