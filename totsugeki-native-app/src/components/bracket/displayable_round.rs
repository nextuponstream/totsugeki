//! Display some round
#![allow(non_snake_case)]

use crate::components::bracket::displayable_match::DisplayMatch;
use crate::DisplayableMatch;
use dioxus::prelude::*;

pub(crate) fn Round(cx: Scope, round: Vec<DisplayableMatch>) -> Element {
    cx.render(rsx!(
        div {
            class: "grid grid-cols-1",
            round.iter().map(|m| DisplayMatch(cx, *m))
        }
    ))
}

/// Lines flow from matches of one round to the next round for a winner bracket
pub(crate) fn WinnerBracketLines(rounds: &[Vec<DisplayableMatch>]) -> Vec<Element> {
    let boxes_in_one_column = rounds[0].len();

    // b belongs in [1; #matches in current round]

    // left line
    // if m X present, draw bottom border of b * 2

    // vertical line
    // if m X present, draw left border of b * 2 + 1 until ???

    // right light
    // if either m X or m X+1 exist, draw bottom border of b * 2

    vec![]
}
