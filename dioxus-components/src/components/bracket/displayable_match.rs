//! Display bracket match in dioxus app

#![allow(non_snake_case)]

use crate::{convert_to_displayable_name, Modal, MAX_NAME_SIZE};
use dioxus::prelude::*;
use totsugeki_display::MinimalMatch;

// TODO find more elegant way to declare constant array of big size
/// Empty string for name to display
pub(crate) const EMPTY_NAME: [u8; MAX_NAME_SIZE] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];
/// Invisible caracter so html element gets displayed
pub(crate) const INVIS_CHAR: &str = "&#8205;";

/// Display a match
pub(crate) fn DisplayMatch(cx: Scope, m: MinimalMatch) -> Element {
    let modal = use_shared_state::<Option<Modal>>(cx).expect("modal to show");

    let start = match m.get_row_hint() {
        Some(h) => format!("row-start-{}", h + 1),
        None => "".into(),
    };
    let players = m.get_players();

    #[allow(clippy::match_like_matches_macro)]
    let is_padding_match = match (&players[0], &players[1]) {
        (p1, p2) if p1.get_name().is_empty() && p2.get_name().is_empty() => true,
        _ => false,
    };
    let outerStyle = if is_padding_match {
        ""
    } else {
        "divide-x divide-y border-1 border-box border hover:bg-gray-300 \
         rounded-md"
    };
    let innerStyle = if is_padding_match { "" } else { "divide-x" };
    // NOTE: removing the invisible caracter does not draw the padding match,
    // which messes up first round bracket display
    let seeds = m.get_seeds();
    let seed1_content = if is_padding_match {
        INVIS_CHAR.into()
    } else {
        seeds[0].to_string()
    };
    let seed2_content = if is_padding_match {
        INVIS_CHAR.into()
    } else {
        seeds[1].to_string()
    };
    let row1 = MatchInRound(
        cx,
        is_padding_match,
        m.clone(),
        seed1_content,
        innerStyle.into(),
        true,
    );
    let row2 = MatchInRound(
        cx,
        is_padding_match,
        m.clone(),
        seed2_content,
        innerStyle.into(),
        false,
    );

    cx.render(rsx!(
        div {
            id: "{m.get_id()}",
            onclick: move |_| {
                if is_padding_match {
                    return;
                }
                let player1 = convert_to_displayable_name(players[0].get_name());
                let player2 = convert_to_displayable_name(players[1].get_name());
                *modal.write() = Some(Modal::EnterMatchResult(m.get_id(), player1, player2));
            },

            class: "col-span-1 flex flex-col my-auto {start} {outerStyle}",
            style: "max-width: 140px;",

            rsx! { row1 }
            rsx! { row2 }
        }
    ))
}

/// Match to display in round
fn MatchInRound(
    cx: Scope,
    is_padding_match: bool,
    m: MinimalMatch,
    seed_content: String,
    innerStyle: String,
    is_player1: bool,
) -> Element {
    let players = m.get_players();
    let player1_name = players[0].get_name();
    let player1 = player1_name.as_str();
    let player2_name = players[1].get_name();
    let player2 = player2_name.as_str();
    let score = m.get_score();
    let score1 = score.0.to_string();
    let score2 = score.1.to_string();
    let (player, score) = if is_player1 {
        (player1, score1.as_str())
    } else {
        (player2, score2.as_str())
    };
    cx.render(rsx!(
            div {
                class: "max-width: 30px; flex flex-row {innerStyle}",
                // TODO format seed display ### so it takes the same space
                // TODO find a way to display padding match without using
                // dangerous_inner_html with the invisible character
                div {
                    // NOTE: as long as seed_content is either the seed or the
                    // invisible html character, then it's fine
                    dangerous_inner_html: "{seed_content}"
                }
                div {
                    class: "{innerStyle} grow pl-1",
                    if is_padding_match {""} else {player}
                }
                div {
                    class: "{innerStyle}",
                    if is_padding_match {""} else {score}
                }
            }
    ))
}
