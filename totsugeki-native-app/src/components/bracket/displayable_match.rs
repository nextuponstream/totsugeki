#![allow(non_snake_case)]

use crate::DisplayableMatch;
use dioxus::prelude::*;
use totsugeki::matches::Id as MatchId;

pub(crate) fn display_match(cx: Scope, m: DisplayableMatch) -> Element {
    let m_id = use_shared_state::<Option<MatchId>>(cx).expect("match id");

    let start = match m.row_hint {
        Some(h) => format!("row-start-{}", h + 1),
        None => "".into(),
    };

    cx.render(rsx!(
        div {
            id: "{m.id}",
            onclick: move |_| {
                *m_id.write() = Some(m.id);
            },

            class: "col-span-1 flex flex-col my-auto box-border border-2 {start} hover:bg-gray-300",

            // TODO format seed display ### so it takes the same space for all
            div {
                class: "grow flex flex-row",
                div { format!("({})", m.seeds[0]) }
                div {
                    class: "box-border border grow",
                    m.player1()
                }
                div {
                    class: "max-width: 15px; box-border border",
                    m.score1()
                }
            }
            div {
                class: "grow flex flex-row",
                div { format!("({})", m.seeds[1]) }
                div {
                    class: "box-border border grow",
                    m.player2()
                }
                div {
                    class: "max-width: 15px; box-border border",
                    m.score2()
                }
            }
        }
    ))
}
