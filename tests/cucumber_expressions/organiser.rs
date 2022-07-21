use crate::get_client;
use crate::World;
use cucumber::{then, when};
use totsugeki::organiser::Organiser;
use totsugeki::DiscussionChannel;
use totsugeki::DiscussionChannelId;
use totsugeki_api_request::organiser::fetch;
use uuid::Uuid;

#[when(expr = "the new bracket originates from discord server of organiser {word}")]
pub fn bracket_originates(w: &mut World, organiser_name: String) {
    w.organiser_name = Some(organiser_name);
}

fn verify_channel_is_active_for_organiser(
    organisers: &Vec<Organiser>,
    organiser_name: &str,
    organiser_id: Uuid,
    discussion_channel_id: DiscussionChannelId,
    bracket_id: Uuid,
) {
    assert!(organisers.len() == 1, "too many results: {:?}", organisers);
    assert!(
        organisers.iter().any(|o| {
            if o.get_organiser_id() == organiser_id
                && o.get_organiser_name().as_str() == organiser_name
            {
                match o.get_active_brackets().get(&discussion_channel_id) {
                    Some(v) => *v == bracket_id,
                    None => {
                        log::error!("no active bracket found for: {}", discussion_channel_id);
                        false
                    }
                }
            } else {
                false
            }
        }),
        "did not find filtered organiser \"{}\" with id: \"{}\". Got:\n{:?}",
        organiser_name,
        organiser_id,
        organisers
    );
}

#[then(expr = "there is a organiser named {word} with the new active bracket")]
async fn organiser_with_active_bracket(w: &mut World, organiser_name: String) {
    let organisers = fetch(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        Some(organiser_name.clone()),
        0,
    )
    .await
    .expect("could not fetch organisers");

    verify_channel_is_active_for_organiser(
        &organisers,
        &organiser_name,
        w.organiser_id.expect("id of organiser"),
        w.discussion_channel_id
            .clone()
            .expect("discussion channel id")
            .get_channel_id()
            .expect("discussion channel id is present"),
        w.bracket_id.expect("bracket id"),
    );
}

#[then(expr = "there is only one organiser with two brackets named {word} and {word}")]
async fn verify_two_brackets_exists_for_organiser(w: &mut World) {
    let organiser_name = &w.organiser_name.clone().expect("organiser name");
    let organisers = fetch(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        Some(organiser_name.clone()),
        0,
    )
    .await
    .expect("could not fetch organisers");
    let organiser_id = w.organiser_id.expect("organiser_id");

    verify_channel_is_active_for_organiser(
        &organisers,
        &organiser_name,
        organiser_id,
        w.discussion_channel_id
            .clone()
            .expect("discussion channel id")
            .get_channel_id()
            .expect("discussion channel id is present"),
        w.bracket_id.expect("bracket id"),
    );

    verify_channel_is_active_for_organiser(
        &organisers,
        &organiser_name,
        organiser_id,
        w.discussion_channel_id
            .clone()
            .expect("discussion channel id")
            .get_channel_id()
            .expect("discussion channel id is present"),
        w.bracket_id.expect("bracket id"),
    );
}
