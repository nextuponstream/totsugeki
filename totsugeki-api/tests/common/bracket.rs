//! bracket domain

use super::TotsugekiApiTestClient;
use crate::common::join::n_players_join_bracket;
use chrono::prelude::*;
use poem::test::{TestJson, TestJsonObject};
use reqwest::StatusCode;
use totsugeki::{
    bracket::{CommandPOST, Id as BracketId, POSTResult, Raw, GET, POST},
    format::Format,
    matches::{Id as MatchId, Match, MatchGET, ReportedResult},
    opponent::Opponent,
    organiser::Id as OrganiserId,
    player::{Id as PlayerId, Player},
    seeding::Method as SeedingMethod,
    DiscussionChannelId,
};
use totsugeki_api::Service;

/// Response after creating a bracket
pub fn parse_bracket_post_response(response: TestJson) -> POSTResult {
    let r = response.value().object();
    let bracket_id_raw = r.get("bracket_id").string();
    let bracket_id = BracketId::parse_str(bracket_id_raw).expect("bracket id");
    let organiser_id_raw = r.get("organiser_id").string();
    let organiser_id = OrganiserId::parse_str(organiser_id_raw).expect("organiser id");
    let discussion_channel_id_raw = r.get("discussion_channel_id").string();
    let discussion_channel_id =
        DiscussionChannelId::parse_str(discussion_channel_id_raw).expect("discussion channel id");

    POSTResult {
        bracket_id,
        organiser_id,
        discussion_channel_id,
    }
}

fn parse_players(response: &TestJsonObject) -> Vec<Player> {
    let players = response.get("players").object_array();
    players
        .iter()
        .map(|p| {
            let name = p.get("name").string().to_string();
            let id = PlayerId::parse_str(p.get("id").string()).expect("player id");
            Player { id, name }
        })
        .collect()
}

/// Response after requesting bracket from id
pub fn parse_bracket_get_response(response: TestJson) -> GET {
    let r = response.value().object();
    let bracket_id_raw = r.get("bracket_id").string();
    let bracket_name = r.get("bracket_name").string();
    let players = parse_players(&r);
    let format = r.get("format").string().to_string();
    let seeding_method = r.get("seeding_method").string().to_string();
    let matches = parse_matches(&r);
    let start_time = r.get("start_time").string().to_string();
    let accept_match_results = r.get("accept_match_results").bool();
    let automatic_match_validation = r.get("automatic_match_validation").bool();
    let barred_from_entering = r.get("barred_from_entering").bool();

    GET {
        bracket_id: BracketId::parse_str(bracket_id_raw).expect("bracket id"),
        bracket_name: bracket_name.to_string(),
        players,
        matches: matches
            .iter()
            .map(|m| {
                let m: MatchGET = (*m).into();
                m
            })
            .collect(),
        format,
        seeding_method,
        start_time,
        accept_match_results,
        automatic_match_validation,
        barred_from_entering,
    }
}

/// Response after requesting brackets
pub fn parse_brackets_get_response(response: TestJson) -> Vec<Raw> {
    let brackets_raw = response.value().object_array();
    brackets_raw
        .iter()
        .map(|o| {
            let bracket_id = o.get("bracket_id").string();
            let bracket_id = BracketId::parse_str(bracket_id).expect("bracket id");
            let bracket_name = o.get("bracket_name").string();
            let players = parse_players(o);
            let format = o.get("format").string().parse::<Format>().expect("format");
            let seeding_method = o
                .get("seeding_method")
                .string()
                .parse::<SeedingMethod>()
                .expect("seeding method");
            let matches = parse_matches(o);
            let start_time = o
                .get("start_time")
                .string()
                .parse::<DateTime<Utc>>()
                .expect("date");
            let accept_match_results = o.get("accept_match_results").bool();
            let automatic_match_validation = o.get("automatic_match_validation").bool();
            let barred_from_entering = o.get("barred_from_entering").bool();
            Raw {
                bracket_id,
                bracket_name: bracket_name.to_string(),
                players: players.iter().map(|p| p.get_id()).collect(),
                player_names: players.iter().map(|p| p.get_name()).collect(),
                matches,
                format,
                seeding_method,
                start_time,
                accept_match_results,
                automatic_match_validation,
                barred_from_entering,
            }
        })
        .collect()
}

/// Parse matches from response
pub fn parse_matches(response: &TestJsonObject) -> Vec<Match> {
    response
        .get("matches")
        .array()
        .iter()
        .map(|m| {
            let m = m.object();
            let id = MatchId::parse_str(m.get("id").string()).expect("id");
            let players = m
                .get("players")
                .string_array()
                .iter()
                .map(|p| p.parse::<Opponent>().expect("opponent"))
                .collect::<Vec<_>>()
                .try_into()
                .expect("2 opponents");
            let seeds = m
                .get("seeds")
                .i64_array()
                .iter()
                .map(|i| *i as usize)
                .collect::<Vec<usize>>()
                .try_into()
                .expect("seeds");
            let winner = m.get("winner").string().parse().expect("winner");
            let looser = m.get("looser").string().parse().expect("looser");

            let reported_results = m.get("reported_results").string_array();
            let rr_1 = reported_results
                .get(0)
                .expect("reported result")
                .parse::<ReportedResult>()
                .expect("parsed reported result");
            let rr_2 = reported_results
                .get(1)
                .expect("reported result")
                .parse::<ReportedResult>()
                .expect("parsed reported result");
            let reported_results = [rr_1.0, rr_2.0];

            Match::try_from(MatchGET::new(
                id,
                players,
                seeds,
                winner,
                looser,
                reported_results,
            ))
            .expect("match")
        })
        .collect::<Vec<Match>>()
}

/// Create bracket. Returns Bracket response, bracket name and organiser name
#[allow(clippy::too_many_arguments)]
pub async fn create_bracket(
    test_api: &TotsugekiApiTestClient,
    organiser_internal_id: &str,
    channel_internal_id: &str,
    service: Service,
    format: Format,
    seeding_method: SeedingMethod,
    start_time: DateTime<Utc>,
    automatic_match_validation: bool,
) -> (POSTResult, String, String) {
    let bracket_name = "weekly".to_string(); // TODO generate name
    let organiser_name = "my-favorite-to".to_string(); // TODO generate name
    let body = POST {
        bracket_name: bracket_name.clone(),
        organiser_name: organiser_name.clone(),
        organiser_internal_id: organiser_internal_id.to_string(),
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service.to_string(),
        format: format.to_string(),
        seeding_method: seeding_method.to_string(),
        start_time: start_time.to_string(),
        automatic_match_validation,
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
    (
        parse_bracket_post_response(resp),
        bracket_name,
        organiser_name,
    )
}

/// Create a bracket with many joining `participants` and start bracket. Returns Bracket and organiser name
#[allow(clippy::too_many_arguments)]
pub async fn players_join_new_bracket_and_bracket_starts(
    test_api: &TotsugekiApiTestClient,
    organiser_internal_id: &str,
    channel_internal_id: &str,
    service: Service,
    format: Format,
    seeding_method: SeedingMethod,
    start_time: DateTime<Utc>,
    participants: usize,
    automatic_match_validation: bool,
) -> (GET, String) {
    let (post_result, _, organiser_name) = create_bracket(
        test_api,
        organiser_internal_id,
        channel_internal_id,
        service,
        format,
        seeding_method,
        start_time,
        automatic_match_validation,
    )
    .await;
    let bracket = n_players_join_bracket(
        test_api,
        participants,
        channel_internal_id,
        service,
        post_result.bracket_id,
    )
    .await;
    tournament_organiser_starts_bracket(test_api, channel_internal_id, service).await;
    (bracket, organiser_name)
}

pub async fn tournament_organiser_starts_bracket(
    test_api: &TotsugekiApiTestClient,
    channel_internal_id: &str,
    service: Service,
) {
    let body_start = CommandPOST {
        channel_internal_id: channel_internal_id.to_string(),
        service_type_id: service.to_string(),
    };
    let res = test_api
        .cli
        .post("/bracket/start")
        .header("X-API-Key", test_api.authorization_header.as_str())
        .body_json(&body_start)
        .send()
        .await;
    res.assert_status(StatusCode::OK);
}

pub async fn get_bracket(test_api: &TotsugekiApiTestClient, bracket_id: BracketId) -> GET {
    let res = test_api
        .cli
        .get(format!("/bracket/{bracket_id}"))
        .send()
        .await;
    res.assert_status(StatusCode::OK);
    let res = res.json().await;
    parse_bracket_get_response(res)
}
