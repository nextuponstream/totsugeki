//! Html building block to display brackets

use super::displayable_round::BoxElement;
use crate::MinimalMatch;
use dioxus::prelude::*;

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
                let left = if b.left_border { "border-l" } else { "" };
                let bottom = if b.bottom_border { "border-b" } else { "" };
                rsx!(div {
                    class: "{left} {bottom}",
                })
            })
        }
    )
}
