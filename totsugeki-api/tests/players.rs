/// test /bracket/players
pub mod common;

use chrono::prelude::*;
use common::{
    bracket::create_bracket, db_types_to_test, join::n_players_join_bracket,
    players::query_players, test_api,
};
use test_log::test;
use totsugeki::{
    format::Format,
    player::{Participants, GET},
    seeding::Method,
};
use totsugeki_api::Service;

#[tokio::test]
async fn empty_bracket_has_no_player() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let internal_discussion_channel_id = "1";
        let service = Service::Discord;
        let (bracket, _, _) = create_bracket(
            &test_api,
            "1",
            internal_discussion_channel_id,
            service,
            Format::default(),
            Method::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        )
        .await;
        let (bracket_id, players) = query_players(
            &test_api,
            &GET {
                internal_discussion_channel_id: internal_discussion_channel_id.to_string(),
                service: service.to_string(),
            },
        )
        .await;
        assert_eq!(bracket_id, bracket.bracket_id);
        assert!(players.is_empty());
    }
}

#[test(tokio::test)]
async fn bracket_returns_information_about_its_players_as_they_join() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let internal_discussion_channel_id = "1";
        let service = Service::Discord;
        let (_, _, _) = create_bracket(
            &test_api,
            "1",
            internal_discussion_channel_id,
            service,
            Format::default(),
            Method::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        )
        .await;
        let body = &GET {
            internal_discussion_channel_id: internal_discussion_channel_id.to_string(),
            service: service.to_string(),
        };
        let (bracket_id, players) = query_players(&test_api, body).await;
        assert!(players.is_empty());

        let bracket = n_players_join_bracket(
            &test_api,
            9,
            internal_discussion_channel_id,
            service,
            bracket_id,
        )
        .await;
        let (_, players) = query_players(&test_api, body).await;
        assert_eq!(players.clone().get_players_list().len(), 9);
        let player_group: Participants = bracket.players.try_into().expect("player group");
        assert!(player_group.have_same_participants(&players));
    }
}
