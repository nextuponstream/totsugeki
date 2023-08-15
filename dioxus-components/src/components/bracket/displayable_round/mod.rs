//! Round to display
#![allow(non_snake_case)]

use crate::components::bracket::displayable_match::DisplayMatch;
use dioxus::prelude::*;
use totsugeki_display::MinimalMatch;

/// Set of matches to display together vertically
pub(crate) fn Round(cx: Scope, round: Vec<MinimalMatch>) -> Element {
    cx.render(rsx!(
        div {
            class: "grid grid-cols-1",
            round.iter().map(|m| DisplayMatch(cx, m.clone()))
        }
    ))
}
