use crate::World;
use cucumber::{then, when};
use totsugeki_api_request::organiser::fetch;

#[when(expr = "the new bracket originates from discord server of organiser {word}")]
fn bracket_originates(w: &mut World, organiser_name: String) {
    w.organiser_name = Some(organiser_name);
}

#[then(expr = "there is a organiser named {word} with the new active bracket")]
fn organiser_with_active_bracket(_w: &mut World, _organiser_name: String) {
    if let Err(_e) = fetch() {}
}
