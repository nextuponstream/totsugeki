//! bracket domain

use poem::test::{TestJson, TestJsonObject};
use totsugeki::{
    bracket::{Bracket, Format, Id as BracketId, POSTResult, GET},
    matches::{Match, MatchGET, Opponent},
    organiser::Id as OrganiserId,
    player::{Id as PlayerId, Player},
    seeding::Method as SeedingMethod,
    DiscussionChannelId,
};

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

    POSTResult::from(bracket_id, organiser_id, discussion_channel_id)
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

    GET {
        bracket_id: BracketId::parse_str(bracket_id_raw).expect("bracket id"),
        bracket_name: bracket_name.to_string(),
        players,
        matches: matches
            .iter()
            .map(|r| {
                r.iter()
                    .map(|m| {
                        let m: MatchGET = m.clone().into();
                        m
                    })
                    .collect()
            })
            .collect(),
        format,
        seeding_method,
    }
}

/// Response after requesting brackets
pub fn parse_brackets_get_response(response: TestJson) -> Vec<Bracket> {
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
            Bracket::from(
                bracket_id,
                bracket_name.to_string(),
                players,
                matches,
                format,
                seeding_method,
            )
        })
        .collect()
}

/// Parse matches from response
pub fn parse_matches(response: &TestJsonObject) -> Vec<Vec<Match>> {
    response
        .get("matches")
        .array()
        .iter()
        .map(|r| {
            let r = r
                .array()
                .iter()
                .map(|m| {
                    let m = m.object();
                    let players = m
                        .get("players")
                        .string_array()
                        .iter()
                        .map(|p| match *p {
                            "BYE" => Opponent::Bye,
                            "?" => Opponent::Unknown,
                            _ => Opponent::Player(PlayerId::parse_str(p).expect("uuid")),
                        })
                        .collect::<Vec<Opponent>>()
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
                    let winner = m.get("winner").string();
                    let winner = Opponent::try_from(winner.to_string()).expect("winner");
                    let looser = m.get("looser").string();
                    let looser = Opponent::try_from(looser.to_string()).expect("looser");

                    Match::from(players, seeds, winner, looser).expect("match")
                })
                .collect::<Vec<Match>>();
            r
        })
        .collect::<Vec<Vec<Match>>>()
}
