/// test /bracket
mod common;

use common::{db_types_to_test, test_api};
use poem::{http::StatusCode, test::TestJson};
use totsugeki::{
    bracket::{Bracket, BracketId, BracketPOST, BracketPOSTResult},
    organiser::OrganiserId,
    DiscussionChannelId, PlayerId,
};

#[tokio::test]
async fn posting_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.post("/bracket").send().await;
        resp.assert_status(StatusCode::UNAUTHORIZED);
        test_api.clean_db().await;
    }
}

fn parse_bracket_post_response(response: TestJson) -> BracketPOSTResult {
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

fn parse_bracket_get_response(response: TestJson) -> Vec<Bracket> {
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

#[tokio::test]
async fn someone_creates_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        // Given my-favorite-to wants to create a bracket named basel-weekly
        let bracket_name = "basel-weekly".to_string(); // TODO generate name
        let organiser_name = "my-favorite-to".to_string();
        let organiser_internal_id = "1".to_string();
        let channel_internal_id = "1".to_string();
        let service_type_id = "discord".to_string();
        let body = BracketPOST::new(
            bracket_name,
            organiser_name,
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

        // Then they search the newly created bracket and find it
        let resp = test_api.cli.get("/bracket/0").send().await;
        resp.assert_status_is_ok();

        let r = resp.json().await;
        let brackets = parse_bracket_get_response(r);

        assert_eq!(
            brackets.len(),
            1,
            "incorrect number of result, expected 1, actual: {}\n{brackets:?}",
            brackets.len()
        );

        assert!(
            brackets
                .iter()
                .any(|b| b.get_id() == bracket_post_resp.get_bracket_id()),
            "no matching bracket id for \"{}\" in:\n {brackets:?}",
            bracket_post_resp.get_bracket_id()
        );

        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn search_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        // Given my-favorite-to wants to create a bracket named zurich-weekly
        let bracket_name = "zurich-weekly".to_string(); // TODO generate name
        let organiser_name = "my-favorite-to".to_string();
        let organiser_internal_id = "1".to_string();
        let channel_internal_id = "1".to_string();
        let service_type_id = "discord".to_string();
        let body = BracketPOST::new(
            bracket_name.clone(),
            organiser_name,
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

        // Then they can filter results and find the created bracket
        let resp = test_api
            .cli
            .get(format!("/bracket/{}/0", bracket_name))
            .send()
            .await;
        resp.assert_status_is_ok();

        let r = resp.json().await;
        let brackets = parse_bracket_get_response(r);

        assert_eq!(
            brackets.len(),
            1,
            "incorrect number of result, expected 1, actual: {}\n{brackets:?}",
            brackets.len()
        );

        assert!(
            brackets
                .iter()
                .any(|b| b.get_id() == bracket_post_resp.get_bracket_id()
                    && b.get_bracket_name() == bracket_name),
            "no matching bracket id for \"{}\" in:\n {brackets:?}",
            bracket_post_resp.get_bracket_id()
        );

        test_api.clean_db().await;
    }
}
