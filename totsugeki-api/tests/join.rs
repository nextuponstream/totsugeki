// test /join endpoint

pub mod common;

use chrono::prelude::*;
use common::{
    bracket::{
        parse_bracket_get_response, parse_bracket_post_response, parse_brackets_get_response,
    },
    db_types_to_test,
    next_match::assert_next_matches,
    test_api,
};
use poem::http::StatusCode;
use std::collections::HashSet;
use test_log::test;
use totsugeki::{
    bracket::{Format, Id as BracketId, POST},
    join::{POSTRequestBody, POSTResponseBody},
    matches::NextMatchGETRequest,
    organiser::Id as OrganiserId,
    player::Id as PlayerId,
    seeding::Method as SeedingMethod,
};
use totsugeki_api::Service;
use tracing::{info, trace};

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
        let service_type_id = Service::Discord.to_string();
        let format = Format::SingleElimination.to_string();
        let seeding_method = SeedingMethod::Strict.to_string();
        let start_time = Utc.ymd(2000, 1, 1).and_hms(0, 0, 0).to_string();
        let body = POST {
            bracket_name: bracket_name.clone(),
            organiser_name,
            organiser_internal_id,
            channel_internal_id: channel_internal_id.clone(),
            service_type_id: service_type_id.clone(),
            format,
            seeding_method,
            start_time,
            automatic_match_validation: false,
        };

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
            let body = POSTRequestBody::new(
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

            // When a player joins, matches get updated automatically
            let res = test_api
                .cli
                .get(format!("/bracket/{}", bracket_post_resp.get_bracket_id()))
                .send()
                .await;
            res.assert_status_is_ok();
            let r = res.json().await;
            let bracket = parse_bracket_get_response(r);
            match i {
                1 | 2 => assert!(bracket.matches.is_empty(), "no matches should be generated when there is not enough players to run a bracket"),
                // There should be only 2 matches since BYE opponent are not relevant for players
                3 => assert_eq!(
                    bracket.matches.len(),
                    2,
                    "Matches where not generated for 3 players: {bracket:?}"
                ),
                _ => {}
            }
        }

        // Then there is enough people for an 8 participant bracket
        let resp = test_api.cli.get("/brackets/0".to_string()).send().await;
        resp.assert_status_is_ok();

        let r = resp.json().await;
        let brackets = parse_brackets_get_response(r);

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
                    && b.get_player_ids().len() == 8
                    // https://stackoverflow.com/a/46767732
                    && b.get_player_ids().iter().all(|p| unique_players.insert(p))
            }),
            "no matching bracket id for \"{}\" in:\n {brackets:?}",
            bracket_post_resp.get_bracket_id()
        );

        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn using_service_to_search_for_next_opponent_needs_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let res = test_api.cli.get("/next_match").send().await;
        res.assert_status(StatusCode::UNAUTHORIZED);
    }
}

#[test(tokio::test)]
async fn bracket_initial_next_opponent_are_correct() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        // Given my-favorite-to has created a bracket named weekly
        let bracket_name = "weekly".to_string(); // TODO generate name
        let organiser_name = "my-favorite-to".to_string();
        let organiser_internal_id = "1".to_string();
        let channel_internal_id = "1".to_string();
        let service_type_id = Service::Discord;
        let format = Format::SingleElimination.to_string();
        let seeding_method = SeedingMethod::Strict.to_string();
        let start_time = Utc.ymd(2000, 1, 1).and_hms(0, 0, 0).to_string();
        let body = POST {
            bracket_name: bracket_name.clone(),
            organiser_name,
            organiser_internal_id,
            channel_internal_id: channel_internal_id.clone(),
            service_type_id: service_type_id.to_string(),
            format,
            seeding_method,
            start_time,
            automatic_match_validation: false,
        };

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
        let mut players: Vec<PlayerId> = vec![];
        for i in 1..=8 {
            let player_internal_id = i.to_string();
            let player_name = format!("player_{i}");
            info!("{player_name} wants to join");
            let channel_internal_id = channel_internal_id.clone();
            let body = POSTRequestBody::new(
                player_internal_id.clone(),
                player_name.clone(),
                channel_internal_id.clone(),
                service_type_id.to_string(),
            );

            let join_resp = test_api
                .cli
                .post("/join")
                .header("X-API-Key", test_api.authorization_header.as_str())
                .body_json(&body)
                .send()
                .await;
            join_resp.assert_status_is_ok();
            let join_resp = join_resp.json().await;
            let join_resp = join_resp.value().object();
            let join_resp = POSTResponseBody {
                player_id: PlayerId::try_from(join_resp.get("player_id").string())
                    .expect("player id"),
                bracket_id: BracketId::try_from(join_resp.get("bracket_id").string())
                    .expect("bracket id"),
                organiser_id: OrganiserId::try_from(join_resp.get("organiser_id").string())
                    .expect("organiser"),
            };
            players.push(join_resp.player_id);
            info!(
                "{player_name} has joined bracket. They have ID: {}",
                join_resp.player_id
            );

            // When a player joins, matches get updated automatically
            let res = test_api
                .cli
                .get(format!("/bracket/{}", bracket_post_resp.get_bracket_id()))
                .send()
                .await;
            res.assert_status_is_ok();
            let r = res.json().await;
            let _bracket = parse_bracket_get_response(r);
            match i {
                1 | 2 => {
                    // When bracket is too small, no matches are generated.
                    // Then response is 404 with a message
                    trace!(
                        "{player_name} does not have a next match since no matches are generated"
                    );
                    let body = NextMatchGETRequest {
                        player_internal_id,
                        channel_internal_id,
                        service_type_id: service_type_id.to_string(),
                    };
                    let res = test_api
                        .cli
                        .get("/next_match")
                        .header("X-API-Key", test_api.authorization_header.as_str())
                        .body_json(&body)
                        .send()
                        .await;
                    res.assert_status(StatusCode::NOT_FOUND);
                    res.assert_text("There is no match for you to play.").await;
                    trace!("Expected answer was that match is not found (404)");
                }
                3 => {
                    assert_next_matches(
                        &[1],
                        &[(2, 3)],
                        &players,
                        &test_api,
                        channel_internal_id.clone(),
                        service_type_id,
                    )
                    .await;
                }
                4 => {
                    assert_next_matches(
                        &[],
                        &[(1, 4), (2, 3)],
                        &players,
                        &test_api,
                        channel_internal_id.clone(),
                        service_type_id,
                    )
                    .await;
                }
                5 => {
                    assert_next_matches(
                        &[1],
                        &[(2, 3), (4, 5)],
                        &players,
                        &test_api,
                        channel_internal_id.clone(),
                        service_type_id,
                    )
                    .await;
                }
                6 => {
                    assert_next_matches(
                        &[1, 2],
                        &[(3, 6), (4, 5)],
                        &players,
                        &test_api,
                        channel_internal_id.clone(),
                        service_type_id,
                    )
                    .await;
                }
                7 => {
                    assert_next_matches(
                        &[1],
                        &[(2, 7), (3, 6), (4, 5)],
                        &players,
                        &test_api,
                        channel_internal_id.clone(),
                        service_type_id,
                    )
                    .await;
                }
                8 => {
                    assert_next_matches(
                        &[],
                        &[(1, 8), (2, 7), (3, 6), (4, 5)],
                        &players,
                        &test_api,
                        channel_internal_id.clone(),
                        service_type_id,
                    )
                    .await;
                }
                _ => unreachable!(),
            }
        }

        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn reporting_result_using_service_is_protected() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let res = test_api.cli.post("/bracket/report").send().await;
        res.assert_status(StatusCode::UNAUTHORIZED);
    }
}
