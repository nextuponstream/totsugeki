/// test /bracket
pub mod common;

use chrono::prelude::*;
use common::{
    bracket::{create_bracket, parse_brackets_get_response},
    db_types_to_test, test_api,
};
use poem::http::StatusCode;
use totsugeki::{format::Format, seeding::Method as SeedingMethod};

#[tokio::test]
async fn removing_player_from_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let res = test_api.cli.post("/bracket/remove").send().await;
        res.assert_status(StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn quitting_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let res = test_api.cli.post("/bracket/quit").send().await;
        res.assert_status(StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn forfeiting_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let res = test_api.cli.post("/bracket/forfeit").send().await;
        res.assert_status(StatusCode::UNAUTHORIZED);
    }
}

#[tokio::test]
async fn validating_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.post("/bracket/validate/1234").send().await;
        resp.assert_status(StatusCode::UNAUTHORIZED);
        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn seeding_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.post("/bracket/seed").send().await;
        resp.assert_status(StatusCode::UNAUTHORIZED);
        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn closing_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.post("/bracket/close").send().await;
        resp.assert_status(StatusCode::UNAUTHORIZED);
        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn starting_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.post("/bracket/start").send().await;
        resp.assert_status(StatusCode::UNAUTHORIZED);
        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn posting_bracket_requires_authorization() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.post("/bracket").send().await;
        resp.assert_status(StatusCode::UNAUTHORIZED);
        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn someone_creates_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_internal_id = "1";
        let channel_internal_id = "1";
        let format = Format::SingleElimination;
        let seeding_method = SeedingMethod::Strict;
        let (bracket_post_resp, bracket_name, _) = create_bracket(
            &test_api,
            organiser_internal_id,
            channel_internal_id,
            totsugeki_api::Service::Discord,
            format,
            seeding_method,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        )
        .await;

        // Then they search the newly created bracket and find it
        let resp = test_api.cli.get("/brackets/0").send().await;
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
            brackets
                .iter()
                .any(|b| b.bracket_id == bracket_post_resp.bracket_id),
            "no matching bracket id for \"{}\" in:\n {brackets:?}",
            bracket_post_resp.bracket_id
        );

        // Retrieving directly works too
        let resp = test_api
            .cli
            .get(format!("/bracket/{}", bracket_post_resp.bracket_id))
            .send()
            .await;
        resp.assert_status_is_ok();

        let r = resp.json().await;
        let r = r.value().object();
        assert_eq!(
            r.get("bracket_id").string(),
            bracket_post_resp.bracket_id.to_string()
        );
        assert_eq!(r.get("bracket_name").string(), bracket_name);
        assert_eq!(r.get("format").string(), format.to_string());
        assert_eq!(r.get("seeding_method").string(), seeding_method.to_string());

        assert_eq!(
            brackets.len(),
            1,
            "incorrect number of result, expected 1, actual: {}\n{brackets:?}",
            brackets.len()
        );

        test_api.clean_db().await;
    }
}

#[tokio::test]
async fn search_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;

        let organiser_internal_id = "1";
        let channel_internal_id = "1";
        let format = Format::SingleElimination;
        let seeding_method = SeedingMethod::Strict;
        let (bracket_post_resp, bracket_name, _) = create_bracket(
            &test_api,
            organiser_internal_id,
            channel_internal_id,
            totsugeki_api::Service::Discord,
            format,
            seeding_method,
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        )
        .await;

        // Then they can filter results and find the created bracket
        let resp = test_api
            .cli
            .get(format!("/brackets/{}/0", bracket_name))
            .send()
            .await;
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
            brackets
                .iter()
                .any(|b| b.bracket_id == bracket_post_resp.bracket_id
                    && b.bracket_name == bracket_name),
            "no matching bracket id for \"{}\" in:\n {brackets:?}",
            bracket_post_resp.bracket_id
        );

        test_api.clean_db().await;
    }
}
