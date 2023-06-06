//! Visualize general details of bracket and view of the bracket
#![allow(non_snake_case)]

use crate::{
    components::bracket::double_elimination_bracket_view::View as DoubleEliminationBracketView,
    components::bracket::single_elimination_bracket_view::View as SingleEliminationBracketView,
    components::Submit,
};
use chrono::prelude::*;
use dioxus::prelude::*;
use totsugeki::{bracket::Bracket, format::Format};

/// Display bracket name, number of players and players
pub fn GeneralDetails(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    let details = bracket.read().to_string();
    let format = bracket.read().get_format().to_string();
    let participants = bracket.read().get_participants();
    let n = participants.len();

    cx.render(rsx!(div {
        h1 {
            class: "text-lg",
            "General details"
        }
        p { details }
        p {
            label { class: "pr-2", "Format:" }
            format
        }
        p {
            label { class: "pr-2", "Players:" }
            n.to_string()
        }
        for p in participants.get_players_list().iter() {
            p { p.to_string() }
        }
    }))
}

/// Update bracket format and rename bracket
pub fn UpdateBracketDetails(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    cx.render(rsx!(

        h2 {
            class: "text-lg",
            "Update bracket"
        }
        form {
            prevent_default: "submit",
            onsubmit: move |event| { update_general_details_of_bracket(bracket, event ) },

            div {
                class: "pb-2",
               label { "Name" }
                input {
                    class: "border border-gray-300 text-sm rounded-lg \
                            focus:ring-blue-500 block p-2.5 \
                            focus:border-blue-500",
                    name: "name",
                }
            }

            div {
                class: "pb-2",
                label {
                    class: "pr-2",
                    "Format"
                }
                select {
                    name: "format",
                    option { "double-elimination" }
                    option { "single-elimination" }
                }
            }

            Submit {}
        }
    ))
}

/// Update bracket name and/or format. When bracket format is updated, thrashes
/// existing bracket matches and regenerate matches for new format.
// FIXME do not regenerate bracket when updating name
fn update_general_details_of_bracket(bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    // let name = e.values.get("name").expect("name").first().expect("");
    // let format = e.values.get("format").expect("format").first().expect("f");
    let name = e.values.get("name").expect("name");
    let format = e.values.get("format").expect("format");
    let is_valid = true;
    let (format, is_valid) = match format.parse::<Format>() {
        Ok(f) => (f, is_valid),
        Err(_e) => (Format::default(), false),
    };

    if !is_valid {
        return;
    }

    *bracket.write() = Bracket::new(
        name,
        format,
        totsugeki::seeding::Method::Strict,
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        true,
    );
}

/// Dioxus component for interactive bracket, using dioxus shared state
/// `Bracket`
pub fn View(cx: Scope) -> Element {
    let format = match use_shared_state::<Bracket>(cx) {
        Some(bracket) => bracket.read().get_format(),
        None => Bracket::default().get_format(),
    };
    let view = match format {
        Format::SingleElimination => SingleEliminationBracketView(cx),
        Format::DoubleElimination => DoubleEliminationBracketView(cx),
    };

    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Bracket view"
        }
        view

    ))
}
