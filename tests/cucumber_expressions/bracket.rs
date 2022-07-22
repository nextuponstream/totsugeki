use crate::{get_client, World};
use cucumber::{given, then, when};
use serenity::model::id::ChannelId;
use totsugeki::DiscussionChannel;
use totsugeki_api_request::bracket::create;
use totsugeki_api_request::bracket::fetch;
use totsugeki_discord_bot::DiscordChannel;

#[given(expr = "{word} wants to create(s|) a bracket named {word}")]
pub fn someone_wants_to_create_bracket(w: &mut World, user: String, bracket_name: String) {
    w.user = Some(user);
    w.bracket_name = Some(bracket_name);
    w.discussion_channel_id = Some(DiscordChannel::new(None, ChannelId(1_u64)));
    w.organiser_internal_id = Some("1".to_string());
    w.organiser_name = Some("FancyBar".to_string());
}

#[when(regex = r"^(?:he|she|they) create(s|) a bracket using discord bot")]
async fn create_bracket_using_discord_bot(w: &mut World) {
    match create(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        w.authorization_header
            .clone()
            .expect("authorization header")
            .as_str(),
        w.bracket_name.clone().expect("bracket name").as_str(),
        w.organiser_name.clone().expect("organiser name").as_str(),
        w.organiser_internal_id
            .clone()
            .expect("organiser internal id")
            .as_str(),
        w.discussion_channel_id
            .clone()
            .expect("discussion channel id"),
    )
    .await
    {
        Ok(r) => {
            let internal_id = w
                .discussion_channel_id
                .clone()
                .expect("discussion channel")
                .get_internal_id();
            w.bracket_id = Some(r.get_bracket_id());
            w.organiser_id = Some(r.get_organiser_id());
            w.discussion_channel_id = Some(DiscordChannel::new(
                Some(r.get_discussion_channel_id()),
                internal_id,
            ))
        }
        Err(e) => panic!("bracket could not be created: {e}"),
    }
}

#[then(
    regex = r"^(?:he|she|they) search the newly created bracket with the discord bot and find it"
)]
async fn see_bracket(w: &mut World) {
    let brackets = fetch(get_client(true), w.tournament_server_addr.as_str(), None, 0)
        .await
        .expect("could not fetch brackets");
    let bracket_name = w.bracket_name.clone().expect("no bracket name");
    assert!(
        brackets
            .into_iter()
            .any(|b| b.get_bracket_name() == bracket_name),
        "did not find \"{}\"",
        bracket_name
    );
}

#[then(regex = r"^(?:he|she|they) can filter results and find the created bracket")]
async fn find_bracket(w: &mut World) {
    let bracket_name = w.bracket_name.clone().expect("no bracket name");
    let brackets = fetch(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        Some(bracket_name.clone()),
        0,
    )
    .await;
    let brackets = brackets.expect("no brackets received");
    assert!(
        brackets
            .clone()
            .into_iter()
            .any(|b| b.get_bracket_name() == bracket_name),
        "did not find filtered bracker \"{}\"",
        bracket_name
    );
    assert!(brackets.len() >= 1, "too many results");
}

#[when(expr = "the organiser {word} has already ran a bracket named {word}")]
pub async fn already_ran_bracket(w: &mut World, _organiser_name: String, bracket_name: String) {
    let bracket_name_to_run = w.bracket_name.clone().expect("bracket name to run");
    someone_wants_to_create_bracket(w, "my-favorite-to".to_string(), bracket_name);
    create_bracket_using_discord_bot(w).await;
    w.bracket_name = Some(bracket_name_to_run);
}
