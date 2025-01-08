//! Forfeit while bracket is running command

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki_core::format::Format;
use totsugeki_core::opponent::Opponent;
use tracing::{info, span, warn, Level};

#[command]
#[description = "Forfeit in running bracket"]
async fn forfeit(ctx: &Context, msg: &Message) -> CommandResult {
    let span = span!(Level::INFO, "Forfeit bracket");
    span.in_scope(|| async {
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let users_copy = users.clone();
        let Some(player) = users_copy.get(&user_id) else {
            warn!("Unregistered user");
            msg.reply(ctx, "You are not registered").await?;
            return Ok::<CommandResult, CommandError>(Ok(()));
        };

        let mut new_matches_message = String::new();
        let (data, new_playable_matches) = match format {
            Format::SingleEliminationBracket => {
                let (seb, new_playable_matches) =
                    single_elimination_bracket.disqualify_participant_from_bracket(player.get_id());
                *bracket_data = (
                    format,
                    users.clone(),
                    seb.clone(),
                    double_elimination_bracket,
                );
                (
                    Data {
                        format,
                        users,
                        single_elimination_bracket: Some(seb),
                        double_elimination_bracket: None,
                    },
                    new_playable_matches,
                )
            }
            Format::DoubleEliminationBracket => {
                let (deb, new_playable_matches) = match double_elimination_bracket
                    .disqualify_participant_from_bracket(player.get_id())
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
                        format,
                        users,
                        single_elimination_bracket: None,
                        double_elimination_bracket: Some(deb),
                    },
                    new_playable_matches,
                )
            }
        };
        if let Some(new_playable_matches) = new_playable_matches {
            for m in new_playable_matches {
                let player1 = match m.get_players()[0] {
                    Opponent(Some(p)) => p,
                    Opponent(None) => panic!("cannot parse opponent"),
                };
                let player2 = match m.get_players()[1] {
                    Opponent(Some(p)) => p,
                    Opponent(None) => panic!("cannot parse opponent"),
                };
                new_matches_message =
                    format!("{}\n{} VS {}", new_matches_message, player1, player2);
            }
        } else {
            new_matches_message = "No new playable matches".into();
        }

        let j = serde_json::to_string(&data).expect("bracket");

        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access
        let l: u64 = u64::try_from(j.len())?;
        f.set_len(l)?; // very important: if output has less chars than previous, output is padded
        f.write_all(j.as_bytes())?;

        info!("{player} forfeited");
        msg.reply(
            ctx,
            format!("You have declared forfeit as {player}.{new_matches_message}"),
        )
        .await?;
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
