use crate::{get_client, World};
use cucumber::{given, then, when};
use totsugeki_api_request::bracket::create;
use totsugeki_api_request::bracket::fetch;

#[given(expr = "{word} wants to create(s|) a bracket named {word}")]
pub fn someone_wants_to_create_bracket(w: &mut World, user: String, bracket_name: String) {
    w.user = Some(user);
    w.bracket_name = Some(bracket_name);
}

#[when(regex = r"^(?:he|she|they) create(s|) a bracket using discord bot")]
async fn create_bracket(w: &mut World) {
    if let Err(e) = create(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        w.authorization_header.as_str(),
        w.bracket_name
            .clone()
            .expect("no bracket name provided")
            .as_str(),
    )
    .await
    {
        panic!("bracket could not be created: {e}");
    }
}

#[then(
    regex = r"^(?:he|she|they) search the newly created bracket with the discord bot and find it"
)]
async fn see_bracket(w: &mut World) {
    let brackets = fetch(get_client(true), w.tournament_server_addr.as_str(), None, 0)
        .await
        .expect("could not fetch brackets");
    let bracket_name = w.bracket_name.clone().expect("no bracket name");
    assert!(
        brackets
            .into_iter()
            .any(|b| b.get_bracket_name() == bracket_name),
        "did not find \"{}\"",
        bracket_name
    );
}

#[then(regex = r"^(?:he|she|they) can filter results and find the created bracket")]
async fn find_bracket(w: &mut World) {
    let bracket_name = w.bracket_name.clone().expect("no bracket name");
    let brackets = fetch(
        get_client(w.accept_invalid_certs),
        w.tournament_server_addr.as_str(),
        Some(bracket_name.clone()),
        0,
    )
    .await;
    let brackets = brackets.expect("no brackets received");
    assert!(
        brackets
            .clone()
            .into_iter()
            .any(|b| b.get_bracket_name() == bracket_name),
        "did not find filtered bracker \"{}\"",
        bracket_name
    );
    assert!(brackets.len() == 1, "too many results");
}
