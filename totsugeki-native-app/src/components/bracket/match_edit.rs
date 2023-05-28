//! Modal component to edit match
#![allow(non_snake_case)]

use crate::{components::Submit, Modal, ShortName};
use dioxus::prelude::*;
use totsugeki::{bracket::Bracket, matches::Id as MatchId, opponent::Opponent};

#[derive(PartialEq, Props)]
pub(crate) struct FormProps {
    pub match_id: MatchId,
    pub player1: String,
    pub player2: String,
}

pub(crate) fn MatchEdit(cx: Scope<FormProps>) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    let result_1 = use_state(cx, || 0);
    let result_2 = use_state(cx, || 0);

    cx.render(rsx!(div {
        form {
            onsubmit: move |event| { update_result(cx, bracket, event) },

            div { "Match ID:" }
            div { "{cx.props.match_id}" }
            div {
                button {
                    onclick: move |_| {
                        result_1.set(2);
                        result_2.set(0);
                    },
                    prevent_default: "onclick",
                    class: "text-white bg-blue-700 hover:bg-blue-800 \
                        focus:ring-4 focus:ring-blue-300 font-medium \
                        rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 \
                        dark:bg-blue-600 dark:hover:bg-blue-700 \
                        focus:outline-none dark:focus:ring-blue-800",
                    "2-0",
                }
                button {
                    onclick: move |_| {
                        result_1.set(2);
                        result_2.set(1);
                    },
                    prevent_default: "onclick",
                    class: "text-white bg-blue-700 hover:bg-blue-800 \
                        focus:ring-4 focus:ring-blue-300 font-medium \
                        rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 \
                        dark:bg-blue-600 dark:hover:bg-blue-700 \
                        focus:outline-none dark:focus:ring-blue-800",
                    "2-1",
                }
                button {
                    onclick: move |_| {
                        result_1.set(1);
                        result_2.set(2);
                    },
                    prevent_default: "onclick",
                    class: "text-white bg-blue-700 hover:bg-blue-800 \
                        focus:ring-4 focus:ring-blue-300 font-medium \
                        rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 \
                        dark:bg-blue-600 dark:hover:bg-blue-700 \
                        focus:outline-none dark:focus:ring-blue-800",
                    "1-2",
                }
                button {
                    onclick: move |_| {
                        result_1.set(0);
                        result_2.set(2);
                    },
                    prevent_default: "onclick",
                    class: "text-white bg-blue-700 hover:bg-blue-800 \
                        focus:ring-4 focus:ring-blue-300 font-medium \
                        rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 \
                        dark:bg-blue-600 dark:hover:bg-blue-700 \
                        focus:outline-none dark:focus:ring-blue-800",
                    "0-2",
                }
            }
            div {
                class: "py-3 flex flex-row justify-between items-center",

                div { "{cx.props.player1}" }
                ResultInput { name: "result_1", value: "{result_1}" }

                ResultInput { name: "result_2", value: "{result_2}" }
                div { "{cx.props.player2}" }
            }

            Submit {}
        }
    }))
}

#[derive(PartialEq, Props)]
pub(crate) struct ResultProps<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

fn ResultInput<'a>(cx: Scope<'a, ResultProps<'a>>) -> Element<'a> {
    cx.render(rsx!(input {
        class: "border border-gray-300 text-sm rounded-lg \
                    focus:ring-blue-500 focus:border-blue-500 block \
                    p-2.5 w-16",
        r#type: "number",
        name: "{cx.props.name}",
        value: "{cx.props.value}",
    }))
}

fn update_result(cx: Scope<FormProps>, bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    let modal = use_shared_state::<Option<Modal>>(cx).expect("modal to show");
    let b = bracket.write().clone();
    let matches = b.get_matches();

    let Some(m) = matches.iter().find(|m| m.get_id() == cx.props.match_id) else {
        return;
    };

    let (p1, p2) = match m.get_players() {
        [Opponent::Player(p1), Opponent::Player(p2)] => (p1, p2),
        _ => return,
    };
    let r1 = e.values.get("result_1").expect("result for p1");
    let r1 = r1.parse::<i8>().unwrap();
    let r2 = e.values.get("result_2").expect("result for p2");
    let r2 = r2.parse::<i8>().unwrap();
    let result = (r1, r2);

    let b = match b.tournament_organiser_reports_result(p1, result, p2) {
        Ok(b) => b.0,
        Err(e) => {
            println!("{e}"); // TODO use a logging library
            return;
        }
    };
    *bracket.write() = b;
    *modal.write() = None;
}

// FIXME when modal is open, cannot tab into the first field right away
pub(crate) fn MatchEditModal(cx: Scope<Props>) -> Element {
    // inspired from: https://www.kindacode.com/article/how-to-create-a-modal-dialog-with-tailwind-css/
    let modal = use_shared_state::<Option<Modal>>(cx).expect("active modal");
    let (match_id, isHidden, player1, player2) = match *modal.read() {
        Some(Modal::EnterMatchResult(m_id, player1, player2)) => (
            m_id,
            "",
            ShortName { value: player1 },
            ShortName { value: player2 },
        ),
        _ => (
            MatchId::new_v4(),
            "hidden",
            ShortName::default(),
            ShortName::default(),
        ),
    };

    cx.render(rsx!(
        div {
            id:"overlay",
            class: "fixed {isHidden} z-40 w-screen h-screen inset-0 \
                    bg-gray-900 bg-opacity-60",
            // close if clicking outside the modal
            onclick: |_| {
                *modal.write() = None;
            },
        }
        div {
            id: "match_edit_modal",
            class: "{isHidden} fixed z-50 top-1/2 left-1/2 -translate-x-1/2 \
                   -translate-y-1/2 w-96 bg-white rounded-md px-8 py-6 \
                   space-y-5 drop-shadow-lg",
            h1 {
                class: "text-2xl font-semibold",
                "Match results"
            }
            div {
                class: "py-5 divide-x-1",
                rsx! {
                    MatchEdit {
                        match_id: match_id,
                        player1: player1.get(),
                        player2: player2.get(),
                    }
                }
            }
            div {
                class: "flex justify-end",
                button {
                    id: "close",
                    onclick: |_| {
                        *modal.write() = None;
                    },
                    class: "px-5 py-2 bg-blue-500 hover:bg-blue-700 \
                            cursor-pointer rounded-md",
                    "close"
                }
            }
        }
    ))
}

#[derive(PartialEq, Props)]
pub(crate) struct Props {
    pub isHidden: bool,
}
