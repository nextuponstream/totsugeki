//! Modal component to edit match
#![allow(non_snake_case)]

use crate::{components::SUBMIT_CLASS, Modal, ShortName};
use dioxus::prelude::*;
use totsugeki::{bracket::Bracket, matches::Id as MatchId, opponent::Opponent};

#[derive(PartialEq, Props)]
/// Props for match edit modal
pub(crate) struct FormProps {
    /// match identifier
    pub match_id: MatchId,
    /// name of player 1
    pub player1: String,
    /// name of player 2
    pub player2: String,
}

/// Component to edit match in modal
pub(crate) fn MatchEdit(cx: Scope<FormProps>) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    let result_1 = use_state(cx, || 0);
    let result_2 = use_state(cx, || 0);

    cx.render(rsx!(div {
        form {
            onsubmit: move |event| { update_bracket_with_match_result(cx, bracket, event) },

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
                RoundWonByPlayer { name: "result_1", value: "{result_1}" }

                RoundWonByPlayer { name: "result_2", value: "{result_2}" }
                div { "{cx.props.player2}" }
            }

            input {
                onclick: move |_| {
                    result_1.set(0);
                    result_2.set(0);
                },
                class: "{SUBMIT_CLASS}",
                r#type: "submit",
            },
        }
    }))
}

#[derive(PartialEq, Props)]
/// Props for input RoundWonByPlayer
pub(crate) struct RoundWonByPlayerProps<'a> {
    /// name of input for form submission
    pub name: &'a str,
    /// value of number of round won for form submission
    pub value: &'a str,
}

/// Input UI element for number of won round by player
fn RoundWonByPlayer<'a>(cx: Scope<'a, RoundWonByPlayerProps<'a>>) -> Element<'a> {
    cx.render(rsx!(input {
        class: "border border-gray-300 text-sm rounded-lg \
                    focus:ring-blue-500 focus:border-blue-500 block \
                    p-2.5 w-16",
        r#type: "number",
        name: "{cx.props.name}",
        value: "{cx.props.value}",
    }))
}

/// Update bracket with match result using input values in match edit modal
fn update_bracket_with_match_result(
    cx: Scope<FormProps>,
    bracket: &UseSharedState<Bracket>,
    e: Event<FormData>,
) {
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
    let Some(r1) = e.values.get("result_1") else {
        // TODO log error
        return;
    };
    let Ok(r1) = r1.parse::<i8>() else {
        // TODO log error
        return;
    };
    let Some(r2) = e.values.get("result_2") else {
        // TODO log error
        return;
    };
    let Ok(r2) = r2.parse::<i8>() else {
        // TODO log error
        return;
    };
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

// TODO nitpick when modal is open, cannot tab into the first field right away
/// Component that becomes visible when dioxus shared state `Modal` is set to
/// variant `Some(Modal::EnterMatchResult)`.
/// When either match results is submitted or user closes modal, Match edit
/// modal gets hidden.
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
/// Props for match edit dioxus component
pub(crate) struct Props {
    /// set to `true` to hide modal
    pub isHidden: bool,
}
