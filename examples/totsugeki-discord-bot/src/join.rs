//! Join bracket

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::format::Format;
use totsugeki::matches::result::MatchFormat;
use totsugeki::player::Player;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::validation::AutomaticMatchValidationMode;
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
        let (format, mut users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let player = match users.get(&user_id) {
            Some(p) => p.clone(),
            None => Player::new(name),
        };
        users.insert(user_id, player.clone());

        let data = match format {
            Format::SingleEliminationBracket => {
                let seeding = single_elimination_bracket.get_seeding();
                let mut players = seeding.get();
                players.push(player.get_id());
                let seeding = match Seeding::new(players) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{e}");
                        msg.reply(ctx, format!("{e}")).await?;
                        return Ok::<CommandResult, CommandError>(Ok(()));
                    }
                };
                let seb = SingleEliminationBracket::create(
                    seeding,
                    AutomaticMatchValidationMode::Flexible, // FIXME should be user provided
                    MatchFormat::ft3(),                     // FIXME should be user provided
                    None,                                   // FIXME should be user provided
                );
                users.insert(user_id, player.clone());
                *bracket_data = (
                    format,
                    users.clone(),
                    seb.clone(),
                    double_elimination_bracket.clone(),
                );
                Data {
                    users,
                    format,
                    single_elimination_bracket: Some(seb),
                    double_elimination_bracket: None,
                }
            }
            Format::DoubleEliminationBracket => {
                let seeding = double_elimination_bracket.get_seeding();
                let mut players = seeding.get();
                players.push(player.get_id());
                let seeding = match Seeding::new(players) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{e}");
                        msg.reply(ctx, format!("{e}")).await?;
                        return Ok::<CommandResult, CommandError>(Ok(()));
                    }
                };
                let deb = DoubleEliminationBracket::create(
                    seeding,
                    AutomaticMatchValidationMode::Flexible, // FIXME should be user provided
                    MatchFormat::ft3(),                     // FIXME should be user provided
                    None,                                   // FIXME should be user provided
                );
                users.insert(user_id, player.clone());
                *bracket_data = (
                    format,
                    users.clone(),
                    single_elimination_bracket.clone(),
                    deb.clone(),
                );
                Data {
                    users,
                    format,
                    single_elimination_bracket: None,
                    double_elimination_bracket: Some(deb),
                }
            }
        };

        let j = serde_json::to_string(&data).expect("bracket");

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
