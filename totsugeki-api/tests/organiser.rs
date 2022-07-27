// test /organiser endpoint

mod common;

use std::collections::{HashMap, HashSet};

use common::{bracket::parse_bracket_post_response, db_types_to_test, test_api};
use poem::test::TestJson;
use totsugeki::{
    bracket::{BracketId, BracketPOST, FinalizedBrackets},
    organiser::{Organiser, OrganiserId},
    DiscussionChannelId,
};

fn parse_organiser_get_response(response: TestJson) -> Vec<Organiser> {
    let organisers_raw = response.value().object_array();
    organisers_raw
        .iter()
        .map(|o| {
            let organiser_id_raw = o.get("organiser_id").string();
            let organiser_id = OrganiserId::parse_str(organiser_id_raw).expect("organiser id");
            let organiser_name = o.get("organiser_name").string();
            let active_brackets_raw = o.get("active_brackets").object();
            let active_brackets: Vec<(DiscussionChannelId, BracketId)> = active_brackets_raw
                .iter()
                .map(|a| {
                    (
                        DiscussionChannelId::parse_str(a.0).expect("discussion channel id"),
                        BracketId::parse_str(a.1.string()).expect("bracket id"),
                    )
                })
                .collect();
            let active_brackets = HashMap::from_iter(active_brackets);
            let finalized_brackets_raw = o.get("finalized_brackets").string_array();
            let finalized_brackets_raw: Vec<BracketId> = finalized_brackets_raw
                .iter()
                .map(|id| BracketId::parse_str(id).expect("bracket id"))
                .collect();
            let mut finalized_brackets: FinalizedBrackets = HashSet::new();
            for b in finalized_brackets_raw {
                finalized_brackets.insert(b);
            }

            Organiser::from(
                active_brackets,
                finalized_brackets,
                organiser_id,
                organiser_name.to_string(),
            )
        })
        .collect()
}

#[tokio::test]
async fn new_organiser_is_generated_when_bracket_is_created_if_unknown() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        // Given FancyBar wants to create a bracket named basel-weekly
        let bracket_name = "basel-weekly".to_string(); // TODO generate name
        let organiser_name = "FancyBar".to_string();
        let organiser_internal_id = "1".to_string();
        let channel_internal_id = "1".to_string();
        let service_type_id = "discord".to_string();
        let body = BracketPOST::new(
            bracket_name,
            organiser_name.clone(),
            organiser_internal_id,
            channel_internal_id,
            service_type_id,
        );

        // When they create a bracket using discord bot
        let resp = test_api
            .cli
            .post("/bracket")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        resp.assert_status_is_ok();

        let resp = resp.json().await;
        let bracket_post_resp = parse_bracket_post_response(resp);

        // Then there is a organiser named FancyBar with the new active bracket
        let resp = test_api.cli.get("/organiser/0").send().await;
        resp.assert_status_is_ok();

        let r = resp.json().await;
        let organisers = parse_organiser_get_response(r);

        assert_eq!(
            organisers.len(),
            1,
            "incorrect number of result, expected 1, actual: {}\n{organisers:?}",
            organisers.len()
        );

        assert!(
            organisers.iter().any(
                |o| o.get_organiser_id() == bracket_post_resp.get_organiser_id()
                    && o.get_organiser_name() == organiser_name.as_str()
            ),
            "no matching organiser id for \"{}\" and name \"{organiser_name}\" in:\n {organisers:?}",
            bracket_post_resp.get_organiser_id()
        );

        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn running_two_brackets_at_the_same_time() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        // Given FancyBar wants to run two brackets at the same time for two different games
        let bracket_name = "weekly-game-1".to_string(); // TODO generate name
        let organiser_name = "FancyBar".to_string();
        let organiser_internal_id = "1".to_string();
        let channel_internal_id = "1".to_string();
        let service_type_id = "discord".to_string();
        let body = BracketPOST::new(
            bracket_name,
            organiser_name.clone(),
            organiser_internal_id.clone(),
            channel_internal_id,
            service_type_id.clone(),
        );

        let bracket_name = "weekly-game-2".to_string(); // TODO generate name
        let channel_internal_id = "2".to_string();
        let body_next_bracket = BracketPOST::new(
            bracket_name,
            organiser_name.clone(),
            organiser_internal_id,
            channel_internal_id.clone(),
            service_type_id,
        );

        // When they create brackets using discord bot
        let resp = test_api
            .cli
            .post("/bracket")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body)
            .send()
            .await;
        resp.assert_status_is_ok();

        let resp = resp.json().await;
        let bracket_post_resp_game_1 = parse_bracket_post_response(resp);

        let resp = test_api
            .cli
            .post("/bracket")
            .header("X-API-Key", test_api.authorization_header.as_str())
            .body_json(&body_next_bracket)
            .send()
            .await;
        resp.assert_status_is_ok();

        let resp = resp.json().await;
        let bracket_post_resp_game_2 = parse_bracket_post_response(resp);

        // Then there is only one organiser with two active brackets
        let resp = test_api.cli.get("/organiser/0").send().await;
        resp.assert_status_is_ok();

        let r = resp.json().await;
        let organisers = parse_organiser_get_response(r);

        assert_eq!(
            organisers.len(),
            1,
            "incorrect number of result, expected 1, actual: {}\n{organisers:?}",
            organisers.len()
        );

        assert!(
            organisers.iter().any(
                |o| o.get_organiser_id() == bracket_post_resp_game_1.get_organiser_id()
                    && o.get_organiser_name() == organiser_name.as_str()
                    && o.get_active_brackets().iter().any(|b|
                        b.0 == &bracket_post_resp_game_1.get_discussion_channel_id()
                        && b.1 == &bracket_post_resp_game_1.get_bracket_id()
                    )
                    && o.get_active_brackets().iter().any(|b|
                        b.0 == &bracket_post_resp_game_2.get_discussion_channel_id()
                        && b.1 == &bracket_post_resp_game_2.get_bracket_id()
                    )
            ),
            "no matching organiser id for weekly \"{}\", weekly-return \"{}\", with organiser name \"{organiser_name}\" in:\n {organisers:?}",
            bracket_post_resp_game_1.get_organiser_id(),
            bracket_post_resp_game_2.get_organiser_id()
        );

        test_api.clean_db().await;
    }
}
