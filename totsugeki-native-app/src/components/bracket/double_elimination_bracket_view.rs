//! View of double elimination bracket

use super::ui_primitives::DisplayStuff;
use crate::components::bracket::displayable_round::winner_bracket_lines::lines;
use crate::components::bracket::displayable_round::Round;
use crate::components::bracket::ui_primitives::RoundWithLines;
use crate::ordering::loser_bracket::reorder as reorder_loser_bracket;
use crate::ordering::winner_bracket::reorder as reorder_winner_bracket;
use crate::partition::loser_bracket::loser_bracket;
use crate::{convert, DisplayableMatch};
use dioxus::prelude::*;
use totsugeki::bracket::double_elimination_bracket::Variant as DoubleEliminationVariant;
use totsugeki::bracket::Bracket;

/// View of a double elimination bracket with interactible elements to update
/// its state
pub(crate) fn View(cx: Scope) -> Element {
    let bracket = match use_shared_state::<Bracket>(cx) {
        Some(bracket_ref) => bracket_ref.read().clone(),
        None => Bracket::default(),
    };
    let dev: DoubleEliminationVariant = bracket.clone().try_into().expect("partition");

    let participants = bracket.get_participants();

    let Ok(wb_rounds_matches) = dev.partition_winner_bracket() else {
        // TODO log error
        return None;
    };
    let mut wb_rounds = vec![];
    for r in wb_rounds_matches {
        let round = r.iter().map(|m| convert(m, &participants)).collect();
        wb_rounds.push(round);
    }
    reorder_winner_bracket(&mut wb_rounds);
    let lines = lines(wb_rounds.clone());
    let mut lb_lines = vec![];
    for _ in 0..11 {
        lb_lines.push(vec![]);
    }
    let mut wb_elements: Vec<DisplayStuff> = vec![];

    for (round, round_line) in wb_rounds.clone().into_iter().zip(lines) {
        wb_elements.push(DisplayStuff::Match(round));
        let (left_col, right_col) = round_line.split_at(round_line.len() / 2);
        wb_elements.push(DisplayStuff::Block(left_col.to_vec()));
        wb_elements.push(DisplayStuff::Block(right_col.to_vec()));
    }
    if !wb_rounds.is_empty() {
        wb_elements.push(DisplayStuff::Match(wb_rounds.into_iter().last().unwrap()));
    }
    let wb_columns = wb_elements.len();

    // TODO extract function for wb + lb Match to DisplayableMatch organised by rounds
    let Ok(lb_rounds_matches) = dev.partition_loser_bracket() else {
        // TODO log error
        return None;
    };
    let mut lb_rounds: Vec<Vec<DisplayableMatch>> = vec![];
    for r in lb_rounds_matches {
        let round = r.iter().map(|m| convert(m, &participants)).collect();
        lb_rounds.push(round);
    }
    reorder_loser_bracket(&mut lb_rounds);
    let mut lb_elements: Vec<DisplayStuff> = vec![];
    // let lb_lines = lines(lb_rounds.clone()); // FIXME get lb_lines the right way

    for (round, round_line) in lb_rounds.clone().into_iter().zip(lb_lines) {
        lb_elements.push(DisplayStuff::Match(round));
        let (left_col, right_col) = round_line.split_at(round_line.len() / 2);
        lb_elements.push(DisplayStuff::Block(left_col.to_vec()));
        lb_elements.push(DisplayStuff::Block(right_col.to_vec()));
    }
    if !lb_rounds.is_empty() {
        lb_elements.push(DisplayStuff::Match(lb_rounds.into_iter().last().unwrap()));
    }
    let lb_columns = lb_elements.len();

    cx.render(rsx!(
        // FIXME uncomment
        div {
            class: "grid grid-rows-1 grid-cols-{wb_columns} flex",
            for e in wb_elements {
                match e {
                    DisplayStuff::Match(round) => rsx! { Round(cx, round) },
                    DisplayStuff::Block(line) => rsx! {
                        RoundWithLines(line)
                    },
                }
            }
        }
        br {}
        br {}
        br {}
        br {}
        div {
            class: "grid grid-rows-1 grid-cols-{lb_columns} flex",
            for e in lb_elements {
                match e {
                    DisplayStuff::Match(round) => rsx! { Round(cx, round) },
                    DisplayStuff::Block(line) => rsx! { "TODO"},
                    // rsx! {
                    //     // RoundWithLines(line)

                    // },
                }
            }
        }
    ))
}
