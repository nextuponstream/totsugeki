//! Validate match as tournament organiser

use crate::{Config, Data};
use std::{io::prelude::*, path::Path};
// use async_fs::File;
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use totsugeki::{matches::Id as MatchId, opponent::Opponent};
use tracing::{info, span, warn, Level};

#[command]
#[description = "Validate match in bracket"]
#[allowed_roles("TO")]
async fn validate(ctx: &Context, msg: &Message, args: Args) -> CommandResult {
    let span = span!(Level::INFO, "Validate match in bracket command");
    span.in_scope(|| async {
        let match_id = args.parse::<MatchId>()?;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (bracket, users) = bracket_data.clone();

        let (bracket, new_matches) = match bracket.clone().validate_match_result(match_id) {
            Ok(r) => r,
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };

        let mut new_matches_message = "".to_string();
        for m in new_matches {
            let player1 = match m.get_players()[0].clone() {
                Opponent::Player(p) => p,
                Opponent::Unknown => panic!("cannot parse opponent"),
            };
            let player2 = match m.get_players()[1].clone() {
                Opponent::Player(p) => p,
                Opponent::Unknown => panic!("cannot parse opponent"),
            };
            new_matches_message = format!(
                "{}\n{} VS {}\n- {}: {}\n- {}: {}",
                new_matches_message,
                player1.get_name(),
                player2.get_name(),
                player1.get_name(),
                player1.get_id(),
                player2.get_name(),
                player2.get_id(),
            );
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

        info!("Match validated in bracket");
        msg.reply(
            ctx,
            format!("You validated {match_id}.{new_matches_message}"),
        )
        .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
