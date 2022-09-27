//! Report result as a player

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki::{matches::ReportedResult, opponent::Opponent};
use tracing::{info, span, warn, Level};

#[command]
#[description = "Report result of your match. Available in the same discussion channel of the active bracket."]
#[usage = "<RESULT (2-0, 0-2, 1-2...)>"]
async fn report(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    // TODO add description example values
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Report bracket command");
    span.in_scope(|| async {
        let reported_result = args.single::<ReportedResult>()?;
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (bracket, users) = bracket_data.clone();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access

        let player = match users.get(&user_id) {
            Some(p) => p.clone(),
            None => {
                warn!("user wants to report but they are not registered");
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };

        let (bracket, _match_id, new_matches) = match bracket
            .clone()
            .report_result(player.get_id(), reported_result.0)
        {
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

        *bracket_data = (bracket, users);
        let j = serde_json::to_string(&bracket_data.clone()).expect("bracket");
        f.write_all(j.as_bytes())?;
        msg.reply(
            ctx,
            format!("You have reported {reported_result}.{new_matches_message}"),
        )
        .await?;
        info!("{player} reported result");

        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}

#[command]
#[description = "Report result of match between two players. Available in the same discussion channel of the active bracket."]
#[usage = "<Player 1 ID> <RESULT (2-0, 0-2, 1-2...)> <Player 2 ID>"]
#[allowed_roles("TO")]
async fn tournament_organiser_reports(
    ctx: &Context,
    msg: &Message,
    mut args: Args,
) -> CommandResult {
    // TODO add description example values
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Report bracket command");
    span.in_scope(|| async {
        let reported_result = args.single::<ReportedResult>()?;
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (bracket, users) = bracket_data.clone();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access

        let player = match users.get(&user_id) {
            Some(p) => p.clone(),
            None => {
                warn!("user wants to report but they are not registered");
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };

        let (bracket, _match_id, new_matches) = match bracket
            .clone()
            .report_result(player.get_id(), reported_result.0)
        {
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

        *bracket_data = (bracket, users);
        let j = serde_json::to_string(&bracket_data.clone()).expect("bracket");
        f.write_all(j.as_bytes())?;
        msg.reply(
            ctx,
            format!("You have reported {reported_result}.{new_matches_message}"),
        )
        .await?;
        info!("{player} reported result");

        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}