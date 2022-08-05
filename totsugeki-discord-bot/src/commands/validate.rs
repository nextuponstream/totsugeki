//! validate match

use crate::{get_client, Api};
use serenity::{
    client::Context,
    framework::standard::{macros::command, Args, CommandResult},
    model::channel::Message,
};
use totsugeki::matches::Id as MatchId;
use totsugeki_api_request::{validate::send, RequestError};

#[command]
#[description = "Validate reported results from player for a match"]
#[usage = "<MATCH ID>"]
#[allowed_roles("TO")]
async fn validate(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let match_id = args.single::<MatchId>()?;
    let tournament_server = {
        let data_read = ctx.data.read().await;
        data_read
            .get::<Api>()
            .expect("Expected TournamentServer in TypeMap.")
            .clone()
    };
    match send(
        get_client(tournament_server.accept_invalid_certs)?,
        tournament_server.get_connection_string().as_str(),
        tournament_server.get_authorization_header().as_str(),
        match_id,
    )
    .await
    {
        Ok(()) => {
            msg.reply(ctx, "Match validated. Bracket updated successfully.")
                .await?;
            Ok(())
        }
        Err(e) => match e {
            RequestError::Request(re, e_msg) => {
                msg.reply(ctx, e_msg.as_str()).await?;
                log::error!("{e_msg}");
                Err(Box::new(RequestError::Request(re, e_msg)))
            }
            RequestError::BracketParsingError(e) => {
                msg.reply(ctx, format!("{e}")).await?;
                log::error!("User could not request bracket: {e}");
                Err(Box::new(RequestError::BracketParsingError(e)))
            }
            RequestError::MatchIdParsingError(e) => {
                msg.reply(ctx, format!("{e}")).await?;
                log::error!("User could not request bracket: {e}");
                Err(Box::new(RequestError::MatchIdParsingError(e)))
            }
            RequestError::NextMatch(e) => {
                msg.reply(ctx, e.to_string()).await?;
                log::error!("User could not request bracket: {e}");
                Err(Box::new(RequestError::NextMatch(e)))
            }
        },
    }
}
