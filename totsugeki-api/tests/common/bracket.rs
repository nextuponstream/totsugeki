//! bracket domain

use poem::test::TestJson;
use totsugeki::{
    bracket::{Bracket, BracketId, BracketPOSTResult},
    organiser::OrganiserId,
    DiscussionChannelId, PlayerId,
};

pub fn parse_bracket_post_response(response: TestJson) -> BracketPOSTResult {
    let r = response.value().object();
    let bracket_id_raw = r.get("bracket_id").string();
    let bracket_id = BracketId::parse_str(bracket_id_raw).expect("bracket id");
    let organiser_id_raw = r.get("organiser_id").string();
    let organiser_id = OrganiserId::parse_str(organiser_id_raw).expect("organiser id");
    let discussion_channel_id_raw = r.get("discussion_channel_id").string();
    let discussion_channel_id =
        DiscussionChannelId::parse_str(discussion_channel_id_raw).expect("discussion channel id");

    BracketPOSTResult::from(bracket_id, organiser_id, discussion_channel_id)
}

pub fn parse_bracket_get_response(response: TestJson) -> Vec<Bracket> {
    let brackets_raw = response.value().object_array();
    brackets_raw
        .iter()
        .map(|o| {
            let bracket_id = o.get("bracket_id").string();
            let bracket_id = BracketId::parse_str(bracket_id).expect("bracket id");
            let bracket_name = o.get("bracket_name").string();
            let players = o.get("players").string_array();
            let players = players
                .iter()
                .map(|p| PlayerId::parse_str(p).expect("player id"))
                .collect();
            Bracket::from(bracket_id, bracket_name.to_string(), players)
        })
        .collect()
}
