//! View of double elimination bracket

use super::ui_primitives::BracketPrimitives;
use crate::components::bracket::displayable_match::DisplayMatch;
use crate::components::bracket::displayable_round::Round;
use crate::components::bracket::match_edit::MatchEditModal;
use crate::components::bracket::ui_primitives::ConnectMatchesBetweenRounds;
use crate::Modal;
use dioxus::prelude::*;
use totsugeki::bracket::double_elimination_variant::Variant as DoubleEliminationVariant;
use totsugeki::bracket::Bracket;
use totsugeki_display::loser_bracket::lines as loser_bracket_lines;
use totsugeki_display::loser_bracket::reorder as reorder_loser_bracket;
use totsugeki_display::winner_bracket::lines as winner_bracket_lines;
use totsugeki_display::winner_bracket::reorder as reorder_winner_bracket;
use totsugeki_display::{from_participants, MinimalMatch};

/// View of a double elimination bracket with interactible elements to update
/// its state
pub(crate) fn View(cx: Scope) -> Element {
    let Some(modal) = use_shared_state::<Option<Modal>>(cx) else {
        log::error!("modal");
        return None;
    };
    let isMatchEditModalHidden = !matches!(*modal.read(), Some(Modal::EnterMatchResult(_, _, _)));

    let bracket = match use_shared_state::<Bracket>(cx) {
        Some(bracket) => bracket.read().clone(),
        None => Bracket::default(),
    };
    let Ok(dev) = std::convert::TryInto::<DoubleEliminationVariant>::try_into(bracket.clone())
    else {
        log::error!("double elimination variant");
        return None;
    };

    let participants = bracket.get_participants();

    let Ok(wb_rounds_matches) = dev.partition_winner_bracket() else {
        log::warn!("winner bracket");
        return None;
    };
    let mut wb_rounds = vec![];
    for r in wb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        wb_rounds.push(round);
    }
    reorder_winner_bracket(&mut wb_rounds);
    let Some(wb_lines) = winner_bracket_lines(&wb_rounds) else {
        log::error!("winner bracket connecting lines");
        return None;
    };
    let mut wb_elements: Vec<BracketPrimitives> = vec![];

    let wb_lines_length = wb_lines.len();
    for (round, round_line) in wb_rounds.clone().into_iter().zip(wb_lines) {
        wb_elements.push(BracketPrimitives::Match(round));
        let (left_col, right_col) = round_line.split_at(round_line.len() / 2);
        wb_elements.push(BracketPrimitives::Block(left_col.to_vec()));
        wb_elements.push(BracketPrimitives::Block(right_col.to_vec()));
    }

    assert_eq!(
        wb_rounds.len() - wb_lines_length,
        1,
        "each round paired with lines except winners finals round"
    );
    wb_elements.push(BracketPrimitives::Match(
        wb_rounds.into_iter().last().expect("winner finals round"),
    ));
    let wb_columns = wb_elements.len();

    // TODO extract function for wb + lb Match to DisplayableMatch organised by rounds
    let Ok(lb_rounds_matches) = dev.partition_loser_bracket() else {
        log::error!("loser bracket");
        return None;
    };
    let mut lb_rounds: Vec<Vec<MinimalMatch>> = vec![];
    for r in lb_rounds_matches {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        lb_rounds.push(round);
    }
    reorder_loser_bracket(&mut lb_rounds);
    let mut lb_elements: Vec<BracketPrimitives> = vec![];
    let Some(lb_lines) = loser_bracket_lines(lb_rounds.clone()) else {
        log::error!("loser bracket connecting lines");
        return None;
    };

    let lb_lines_length = lb_lines.len();
    for (round, round_line) in lb_rounds.clone().into_iter().zip(lb_lines) {
        lb_elements.push(BracketPrimitives::Match(round));
        let (left_col, right_col) = round_line.split_at(round_line.len() / 2);
        lb_elements.push(BracketPrimitives::Block(left_col.to_vec()));
        lb_elements.push(BracketPrimitives::Block(right_col.to_vec()));
    }

    assert_eq!(
        lb_rounds.len() - lb_lines_length,
        1,
        "each round paired with lines except losers finals round"
    );
    lb_elements.push(BracketPrimitives::Match(
        lb_rounds.into_iter().last().expect("losers finals round"),
    ));
    let lb_columns = lb_elements.len();

    let Ok((gf, gf_reset)) = dev.grand_finals_and_reset() else {
        log::error!("grand finals+reset");
        return None;
    };
    let gf = from_participants(&gf, &participants);
    let gf_reset = from_participants(&gf_reset, &participants);

    cx.render(rsx!(
        MatchEditModal { isHidden: isMatchEditModalHidden }
        h1 {
            class: "text-lg",
            "Winner bracket",
        }
        div {
            class: "grid grid-rows-1 grid-cols-{wb_columns} flex",
            for e in wb_elements.into_iter() {
                match e {
                    BracketPrimitives::Match(round) => rsx! { Round(cx, round) },
                    BracketPrimitives::Block(line) => rsx! {
                        ConnectMatchesBetweenRounds(line)
                    },
                }
            }
        }
        br {}
        br {}
        br {}
        br {}
        h1 {
            class: "text-lg",
            "Loser bracket",
        }
        div {
            class: "grid grid-rows-1 grid-cols-{lb_columns} flex",
            for e in lb_elements.into_iter() {
                match e {
                    BracketPrimitives::Match(round) => rsx! { Round(cx, round) },
                    BracketPrimitives::Block(line) =>
                    rsx! {
                        ConnectMatchesBetweenRounds(line)

                    },
                }
            }
        }
        h1 {
            class: "text-lg",
            "Grand finals",
        }
        DisplayMatch(cx, gf)
        h1 {
            class: "text-lg",
            "Grand final reset",
        }
        DisplayMatch(cx, gf_reset)
    ))
}
