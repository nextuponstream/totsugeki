// test /join endpoint

pub mod common;

use std::collections::HashSet;

use common::{
    bracket::{parse_bracket_get_response, parse_bracket_post_response},
    db_types_to_test, test_api,
};
use poem::http::StatusCode;
use totsugeki::{bracket::BracketPOST, join::JoinPOSTRequestBody};

#[tokio::test]
async fn joining_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.post("/join").send().await;
        resp.assert_status(StatusCode::UNAUTHORIZED);
        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn players_join_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        // Given my-favorite-to has created a bracket named weekly
        let bracket_name = "weekly".to_string(); // TODO generate name
        let organiser_name = "my-favorite-to".to_string();
        let organiser_internal_id = "1".to_string();
        let channel_internal_id = "1".to_string();
        let service_type_id = "discord".to_string();
        let body = BracketPOST::new(
            bracket_name.clone(),
            organiser_name,
            organiser_internal_id,
            channel_internal_id.clone(),
            service_type_id.clone(),
        );

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

        // When many players join
        for i in 1..=8 {
            let player_internal_id = i.to_string();
            let player_name = format!("player_{i}");
            let channel_internal_id = channel_internal_id.clone();
            let service_type_id = service_type_id.clone();
            let body = JoinPOSTRequestBody::new(
                player_internal_id,
                player_name,
                channel_internal_id,
                service_type_id,
            );

            let resp = test_api
                .cli
                .post("/join")
                .header("X-API-Key", test_api.authorization_header.as_str())
                .body_json(&body)
                .send()
                .await;
            resp.assert_status_is_ok();
        }

        // Then there is enough people for an 8 participant tournament
        let resp = test_api.cli.get(format!("/bracket/0")).send().await;
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
            brackets.iter().any(|b| {
                let mut unique_players = HashSet::new();

                b.get_id() == bracket_post_resp.get_bracket_id()
                    && b.get_bracket_name() == bracket_name
                    && b.get_players().len() == 8
                    // https://stackoverflow.com/a/46767732
                    && b.get_players().iter().all(|p| unique_players.insert(p))
            }),
            "no matching bracket id for \"{}\" in:\n {brackets:?}",
            bracket_post_resp.get_bracket_id()
        );

        test_api.clean_db().await;
    }
}
