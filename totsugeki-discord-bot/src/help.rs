use std::collections::HashSet;

use serenity::{
    client::Context,
    framework::standard::{
        help_commands, macros::help, Args, CommandGroup, CommandResult, HelpOptions,
    },
    model::{channel::Message, id::UserId},
};

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
    let _msg = help_commands::with_embeds(context, msg, args, help_options, groups, owners).await?;
    Ok(())
}
