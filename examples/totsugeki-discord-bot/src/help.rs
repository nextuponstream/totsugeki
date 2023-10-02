//! Help command

use serenity::{
    client::Context,
    framework::standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, id::UserId},
};
use std::collections::HashSet;
use tracing::{span, Level};

#[help]
#[lacking_permissions = "Hide"]
async fn help(
    context: &Context,
    msg: &Message,
    args: Args,
    help_options: &'static HelpOptions,
    groups: &[&'static CommandGroup],
    owners: HashSet<UserId>,
) -> CommandResult {
    // NOTE: workaround since instrument macro conflict with discords
    let span = span!(Level::INFO, "Help command");
    span.in_scope(|| async {
        let _msg =
            help_commands::with_embeds(context, msg, args, help_options, groups, owners).await;
    })
    .await;
    Ok(())
}
