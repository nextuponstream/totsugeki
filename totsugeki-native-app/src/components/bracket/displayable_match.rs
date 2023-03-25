#![allow(non_snake_case)]

use crate::convert_name;
use crate::{DisplayableMatch, Modal};
use dioxus::prelude::*;

// TODO find more elegant way to declare constant array of big size
const EMPTY: [u8; 256] = [
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

pub(crate) fn display_match(cx: Scope, m: DisplayableMatch) -> Element {
    let modal = use_shared_state::<Option<Modal>>(cx).expect("modal to show");

    let start = match m.row_hint {
        Some(h) => format!("row-start-{}", h + 1),
        None => "".into(),
    };

    #[allow(clippy::match_like_matches_macro)]
    let is_padding_match = match (m.players[0], m.players[1]) {
        (p1, p2) if p1 == EMPTY && p2 == EMPTY => true,
        _ => false,
    };
    let outerStyle = if is_padding_match {
        ""
    } else {
        "border-2 hover:bg-gray-300"
    };
    let innerStyle = if is_padding_match { "" } else { "border" };
    // NOTE: removing the invisible caracter does not draw the padding match,
    // which messes up first round bracket display
    let seed1_content = if is_padding_match {
        "&#8205;".into()
    } else {
        m.seeds[0].to_string()
    };
    let seed2_content = if is_padding_match {
        "&#8205;".into()
    } else {
        m.seeds[1].to_string()
    };
    let row1 = MatchRow(
        cx,
        is_padding_match,
        m,
        seed1_content,
        innerStyle.into(),
        true,
    );
    let row2 = MatchRow(
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
                    println!("SKIPPED");
                    return;
                }
                let player1 = convert_name(m.player1().into());
                let player2 = convert_name(m.player2().into());
                *modal.write() = Some(Modal::EnterMatchResult(m.id, player1, player2));
            },

            class: "col-span-1 flex flex-col my-auto {start} {outerStyle}",

            rsx! { row1 }
            rsx! { row2 }
        }
    ))
}

fn MatchRow(
    cx: Scope,
    is_padding_match: bool,
    m: DisplayableMatch,
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
                class: "grow flex flex-row {innerStyle}",
                // TODO format seed display ### so it takes the same space for all
                // TODO find a way to display padding match without using
                // dangerous_inner_html with the invisible character
                div {
                    // NOTE: as long as seed_content is either the seed or the
                    // invisible html character, then it's fine
                    dangerous_inner_html: "{seed_content}"
                }
                div {
                    class: "box-border {innerStyle} grow",
                    if is_padding_match {""} else {player}
                }
                div {
                    class: "max-width: 15px; box-border {innerStyle}",
                    if is_padding_match {"".into()} else {score}
                }
            }
    ))
}
