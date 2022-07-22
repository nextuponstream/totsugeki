//! join bracket

use super::bracket::already_ran_bracket;
use crate::{get_client, World};
use cucumber::{given, then, when};
use totsugeki_api_request::join::post;

#[given(expr = "{word} has created a bracket named {word}")]
async fn bracket_is_set(w: &mut World, _user: String, bracket_name: String) {
    w.bracket_name = Some(bracket_name.clone());
    already_ran_bracket(w, "".to_string(), bracket_name).await;
}

#[when(expr = "{word}, {word} and {word} other players join")]
async fn players_join(
    w: &mut World,
    new_player_name: String,
    old_player_name: String,
    remaining_players: i32,
) {
    let mut players = Vec::new();
    players.push(new_player_name);
    players.push(old_player_name);
    for i in 0..remaining_players {
        players.push(format!("player{i}"));
    }
    let mut i = 0;
    for player in players.iter() {
        i = i + 1;

        match post(
            get_client(w.accept_invalid_certs),
            w.tournament_server_addr.as_str(),
            w.authorization_header
                .clone()
                .expect("authorization header")
                .as_str(),
            i.to_string().as_str(),
            player,
            w.discussion_channel_id.clone().expect("discussion channel"),
        )
        .await
        {
            Ok(r) => {
                assert!(
                    w.organiser_id.expect("organiser id") == r.organiser_id,
                    "Wrong organiser: {r:?}"
                )
            }
            Err(e) => panic!("players could not join bracket: {e}"),
        }
    }
}

#[then(expr = "there is enough people for an {word} participants tournament")]
async fn check_if_enough_people(w: &mut World, participants: usize) {
    match totsugeki_api_request::bracket::fetch(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.clone().as_str(),
        Some(w.bracket_name.clone().expect("bracket name")),
        0,
    )
    .await
    {
        Ok(brackets) => {
            assert!(
                brackets.len() == 1,
                "there should be only one bracket, found: {:?}",
                brackets
            );
            assert!(
                brackets.iter().any(|b| b.get_bracket_name()
                    == w.bracket_name.clone().expect("bracket name")
                    && b.get_players().len() == participants),
                "did not find bracket \"{}\" with {} participants: {brackets:?}",
                w.bracket_name.clone().expect("bracket name"),
                participants
            )
        }
        Err(e) => panic!("could not fetch brackets: {e}"),
    }
}
