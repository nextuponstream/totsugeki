//! Display bracket match in dioxus app

#![allow(non_snake_case)]

use crate::{convert_to_displayable_name, MAX_NAME_SIZE};
use crate::{MinimalMatch, Modal};
use dioxus::prelude::*;

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

    let start = match m.row_hint {
        Some(h) => format!("row-start-{}", h + 1),
        None => "".into(),
    };

    #[allow(clippy::match_like_matches_macro)]
    let is_padding_match = match (m.players[0], m.players[1]) {
        (p1, p2) if p1 == EMPTY_NAME && p2 == EMPTY_NAME => true,
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
    let seed1_content = if is_padding_match {
        INVIS_CHAR.into()
    } else {
        m.seeds[0].to_string()
    };
    let seed2_content = if is_padding_match {
        INVIS_CHAR.into()
    } else {
        m.seeds[1].to_string()
    };
    let row1 = MatchInRound(
        cx,
        is_padding_match,
        m,
        seed1_content,
        innerStyle.into(),
        true,
    );
    let row2 = MatchInRound(
        cx,
        is_padding_match,
        m,
        seed2_content,
        innerStyle.into(),
        false,
    );

    cx.render(rsx!(
        div {
            id: "{m.id}",
            onclick: move |_| {
                if is_padding_match {
                    return;
                }
                let player1 = convert_to_displayable_name(m.player1().into());
                let player2 = convert_to_displayable_name(m.player2().into());
                *modal.write() = Some(Modal::EnterMatchResult(m.id, player1, player2));
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
    let (player, score) = if is_player1 {
        (m.player1(), m.score1())
    } else {
        (m.player2(), m.score2())
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
                    if is_padding_match {"".into()} else {score}
                }
            }
    ))
}
