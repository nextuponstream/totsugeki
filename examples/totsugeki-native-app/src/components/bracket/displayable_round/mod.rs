//! Round to display
#![allow(non_snake_case)]

use crate::components::bracket::displayable_match::DisplayMatch;
use crate::MinimalMatch;
use dioxus::prelude::*;

pub mod loser_bracket_lines;
pub mod winner_bracket_lines;

/// Set of matches to display together vertically
pub(crate) fn Round(cx: Scope, round: Vec<MinimalMatch>) -> Element {
    cx.render(rsx!(
        div {
            class: "grid grid-cols-1",
            round.iter().map(|m| DisplayMatch(cx, *m))
        }
    ))
}

/// Display lines using boxes and their borders
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub(crate) struct BoxElement {
    /// true when left border of box should be visible
    pub(crate) left_border: bool,
    /// true when bottom border of box should be visible
    pub(crate) bottom_border: bool,
}

impl BoxElement {
    /// Box with no borders. Alternative to `default()` to use in constants
    const fn empty() -> Self {
        BoxElement {
            left_border: false,
            bottom_border: false,
        }
    }
}
