//! Html building block to display brackets

use super::displayable_round::BoxWithBorder;
use crate::DisplayableMatch;
use dioxus::prelude::*;

/// Matches and necessary building blocks to display a single elimination
/// bracket
pub(crate) enum DisplayStuff {
    /// Display match
    Match(Vec<DisplayableMatch>),
    /// Padding block with
    Block(Vec<BoxWithBorder>),
}

/// Lines connecting rounds
pub(crate) fn RoundWithLines<'a, 'b>(lines: Vec<BoxWithBorder>) -> LazyNodes<'a, 'b> {
    rsx!(
        div {
            class: "grid grid-cols-1",
            lines.iter().map(|b| {
                let left = if b.left { "border-l" } else { "" };
                let bottom = if b.bottom { "border-b" } else { "" };
                rsx!(div {
                    class: "{left} {bottom}",
                })
            })
        }
    )
}
