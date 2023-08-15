//! Html building block to display brackets

use dioxus::prelude::*;
use totsugeki_display::{BoxElement, MinimalMatch};

/// UI primitives for bracket
pub(crate) enum BracketPrimitives {
    /// Display match
    Match(Vec<MinimalMatch>),
    /// Padding block with
    Block(Vec<BoxElement>),
}

/// Lines connecting rounds
pub(crate) fn ConnectMatchesBetweenRounds<'a, 'b>(lines: Vec<BoxElement>) -> LazyNodes<'a, 'b> {
    rsx!(
        div {
            class: "grid grid-cols-1",
            lines.iter().map(|b| {
                let left = if b.get_left_border() { "border-l" } else { "" };
                let bottom = if b.get_bottom_border() { "border-b" } else { "" };
                rsx!(div {
                    class: "{left} {bottom}",
                })
            })
        }
    )
}
