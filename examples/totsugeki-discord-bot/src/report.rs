//! Report result as a player

use crate::{Config, Data};
use fs4::FileExt;
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandError, CommandResult},
    model::channel::Message,
};
use std::{io::prelude::*, path::Path};
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::format::Format;
use totsugeki::player::PlayerID;
use totsugeki::single_elimination_bracket::progression::ProgressionSEB;
use totsugeki::validation::AutomaticMatchValidationMode;
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
        let reported_result = args.single::<String>()?;
        let reported_result: ReportedResult = match reported_result.parse() {
            Ok(r) => r,
            Err(e) => {
                warn!("{e}");
                msg.reply(ctx, format!("{e}")).await?;
                return Ok::<CommandResult, CommandError>(Ok(()));
            }
        };
        let user_id = msg.author.id;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();

        let Some(player) = users.get(&user_id) else {
            warn!("user wants to report but they are not registered");
            return Ok::<CommandResult, CommandError>(Ok(()));
        };

        let new_matches = match format {
            Format::SingleEliminationBracket => {
                let (single_elimination_bracket_matches, _, new_matches) =
                    match single_elimination_bracket
                        .report_result(player.get_id(), reported_result.0.unwrap())
                    {
                        Ok(t) => t,
                        Err(e) => {
                            warn!("{e}");
                            msg.reply(ctx, format!("{e}")).await?;
                            return Ok::<CommandResult, CommandError>(Ok(()));
                        }
                    };
                *bracket_data = (
                    Format::SingleEliminationBracket,
                    users.clone(),
                    single_elimination_bracket_matches.clone(),
                    double_elimination_bracket.clone(),
                );
                let d = Data {
                    format: Format::SingleEliminationBracket,
                    single_elimination_bracket: Some(single_elimination_bracket_matches),
                    double_elimination_bracket: Some(double_elimination_bracket),
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
                new_matches
            }
            Format::DoubleEliminationBracket => {
                let seeding = double_elimination_bracket.get_seeding();
                let (double_elimination_bracket_matches, _, new_matches) =
                    match double_elimination_bracket
                        .report_result_dangerous(player.get_id(), reported_result.0.unwrap())
                    {
                        Ok(t) => t,
                        Err(e) => {
                            warn!("{e}");
                            msg.reply(ctx, format!("{e}")).await?;
                            return Ok::<CommandResult, CommandError>(Ok(()));
                        }
                    };
                let deb = DoubleEliminationBracket::new(
                    double_elimination_bracket_matches,
                    seeding,
                    AutomaticMatchValidationMode::Flexible, // FIXME should be user provided
                );
                *bracket_data = (
                    Format::DoubleEliminationBracket,
                    users.clone(),
                    single_elimination_bracket.clone(),
                    deb.clone(),
                );
                let d = Data {
                    format: Format::DoubleEliminationBracket,
                    single_elimination_bracket: Some(single_elimination_bracket),
                    double_elimination_bracket: Some(deb),
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
                new_matches
            }
        };
        let mut new_matches_message = String::new();
        for m in new_matches {
            let player1 = match m.get_players()[0] {
                Opponent(Some(p)) => p,
                Opponent(None) => panic!("cannot parse opponent"),
            };
            let player2 = match m.get_players()[1] {
                Opponent(Some(p)) => p,
                Opponent(None) => panic!("cannot parse opponent"),
            };
            new_matches_message = format!("{}\n{} VS {}", new_matches_message, player1, player2);
        }

        info!("{player} reported result");
        msg.reply(
            ctx,
            format!("You have reported {reported_result}.{new_matches_message}"),
        )
        .await?;

        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}

#[command]
#[description = "Report result of match between two players. Available in the same discussion channel of the active bracket."]
#[usage = "<Player 1 ID> <RESULT (2-0, 0-2, 1-2...)> <Player 2 ID>"]
#[aliases("tor")]
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
        let player1 = args.single::<PlayerID>()?;
        let reported_result = args.single::<ReportedResult>()?;
        let player2 = args.single::<PlayerID>()?;

        let data = ctx.data.read().await;
        let config = data.get::<Config>().expect("filename").clone();
        let bracket_data = data.get::<Data>().expect("data").clone();
        let mut bracket_data = bracket_data.write().await;
        let (format, users, single_elimination_bracket, double_elimination_bracket) =
            bracket_data.clone();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .open(Path::new(config.as_ref()))?;
        f.lock_exclusive().expect("lock"); // prevent concurrent access

        let (new_playable_matches, data) = match format {
            Format::SingleEliminationBracket => {
                let (seb, _match_id, new_playable_matches) = match single_elimination_bracket
                    .tournament_organiser_reports_result(
                        player1,
                        reported_result.0.unwrap(),
                        player2,
                    ) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{e}");
                        msg.reply(ctx, format!("{e}")).await?;
                        return Ok::<CommandResult, CommandError>(Ok(()));
                    }
                };
                *bracket_data = (
                    Format::SingleEliminationBracket,
                    users.clone(),
                    seb.clone(),
                    double_elimination_bracket.clone(),
                );
                let data = Data {
                    users: users.clone(),
                    format,
                    single_elimination_bracket: Some(seb),
                    double_elimination_bracket: Some(double_elimination_bracket),
                };
                (new_playable_matches, data)
            }
            Format::DoubleEliminationBracket => {
                let (deb, _match_id, new_playable_matches) = match double_elimination_bracket
                    .tournament_organiser_reports_result_dangerous(
                        player1,
                        reported_result.0.unwrap(),
                        player2,
                    ) {
                    Ok(r) => r,
                    Err(e) => {
                        warn!("{e}");
                        msg.reply(ctx, format!("{e}")).await?;
                        return Ok::<CommandResult, CommandError>(Ok(()));
                    }
                };
                *bracket_data = (
                    Format::DoubleEliminationBracket,
                    users.clone(),
                    single_elimination_bracket.clone(),
                    deb.clone(),
                );
                let data = Data {
                    users: users.clone(),
                    format,
                    single_elimination_bracket: Some(single_elimination_bracket),
                    double_elimination_bracket: Some(deb),
                };
                (new_playable_matches, data)
            }
        };
        let mut new_matches_message = String::new();
        for m in new_playable_matches {
            let player1 = match m.get_players()[0] {
                Opponent(Some(p)) => p,
                Opponent(None) => panic!("cannot parse opponent"),
            };
            let player2 = match m.get_players()[1] {
                Opponent(Some(p)) => p,
                Opponent(None) => panic!("cannot parse opponent"),
            };
            new_matches_message = format!("{}\n{} VS {}", new_matches_message, player1, player2);
        }

        let j = serde_json::to_string(&data).expect("bracket");
        f.write_all(j.as_bytes())?;
        msg.reply(
            ctx,
            format!("You have reported {reported_result}.{new_matches_message}"),
        )
        .await?;
        info!("Reported result");

        // workaround: https://rust-lang.github.io/async-book/07_workarounds/02_err_in_async_blocks.html
        Ok::<CommandResult, CommandError>(Ok(()))
    })
    .await?
}
