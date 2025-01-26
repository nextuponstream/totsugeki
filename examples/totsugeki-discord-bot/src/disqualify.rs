//! Disqualify player from running bracket command

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki_core::format::Format;
use totsugeki_core::ID;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Disqualify player from bracket"]
#[allowed_roles("TO")]
#[aliases("dq")]
async fn disqualify(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let span = span!(Level::INFO, "Disqualify player from bracket");
    span.in_scope(|| async {
        let player_id = args.single::<ID>()?;
        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let new_matches_message = String::new();
        let (data, _new_playable_matches) = match format {
            Format::SingleEliminationBracket => {
                let (seb, new_playable_matches) =
                    single_elimination_bracket.disqualify_participant_from_bracket(player_id);
                *bracket_data = (
                    format,
                    users.clone(),
                    seb.clone(),
                    double_elimination_bracket.clone(),
                );
                (
                    Data {
                        users,
                        format,
                        single_elimination_bracket: Some(seb),
                        double_elimination_bracket: Some(double_elimination_bracket),
                    },
                    new_playable_matches,
                )
            }
            Format::DoubleEliminationBracket => {
                let (deb, new_playable_matches) = match double_elimination_bracket
                    .disqualify_participant_from_bracket(player_id)
                {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{e}");
                        msg.reply(ctx, format!("{e}")).await?;
                        return Ok::<CommandResult, CommandError>(Ok(()));
                    }
                };
                *bracket_data = (
                    format,
                    users.clone(),
                    single_elimination_bracket.clone(),
                    deb.clone(),
                );
                (
                    Data {
                        users,
                        format,
                        single_elimination_bracket: Some(single_elimination_bracket),
                        double_elimination_bracket: Some(deb),
                    },
                    new_playable_matches,
                )
            }
        };

        let j = serde_json::to_string(&data).expect("bracket");

        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has fewer chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("{player_id} disqualified");
        msg.reply(
            ctx,
            format!("{player_id} was disqualified.{new_matches_message}"),
        )
        .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
