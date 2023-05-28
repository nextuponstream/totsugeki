//! Round to display
#![allow(non_snake_case)]

use crate::components::bracket::displayable_match::DisplayMatch;
use crate::DisplayableMatch;
use dioxus::prelude::*;

pub mod loser_bracket_lines;
pub mod winner_bracket_lines;

/// Set of matches to display together vertically
pub(crate) fn Round(cx: Scope, round: Vec<DisplayableMatch>) -> Element {
    cx.render(rsx!(
        div {
            class: "grid grid-cols-1",
            round.iter().map(|m| DisplayMatch(cx, *m))
        }
    ))
}

/// Box that may have a left or bottom border
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct BoxWithBorder {
    pub(crate) left: bool,
    pub(crate) bottom: bool,
}
