//! View of a single elimination bracket
use super::{ui_primitives::BracketPrimitives, ui_primitives::ConnectMatchesBetweenRounds};
use crate::{
    components::bracket::displayable_round::Round, components::bracket::match_edit::MatchEditModal,
    Modal,
};
use dioxus::prelude::*;
use totsugeki::bracket::single_elimination_variant::Variant as SingleEliminationVariant;
use totsugeki::bracket::Bracket;
use totsugeki_display::from_participants;
use totsugeki_display::winner_bracket::lines;
use totsugeki_display::winner_bracket::reorder;

/// View over single elimination bracket
#[allow(dead_code)]
pub(crate) fn View(cx: Scope) -> Element {
    // FIXME problem switching from deb to seb, panics
    let modal = use_shared_state::<Option<Modal>>(cx).expect("modal to show");
    let isMatchEditModalHidden = !matches!(*modal.read(), Some(Modal::EnterMatchResult(_, _, _)));
    let bracket = match use_shared_state::<Bracket>(cx) {
        Some(bracket_ref) => bracket_ref.read().clone(),
        None => Bracket::default(),
    };
    let sev: SingleEliminationVariant = bracket.clone().try_into().expect("partition");

    let participants = bracket.get_participants();

    // let mut rounds = winner_bracket(matches, &participants);
    let match_by_rounds = sev.partition_by_round().expect("rounds");
    let mut rounds = vec![];
    // FIXME find a way to map vec of vec from one type to another
    // Note: did not find a way to map a vec of vec of Match into vec of vec of
    // Displayable match
    for r in match_by_rounds {
        let round = r
            .iter()
            .map(|m| from_participants(m, &participants))
            .collect();
        rounds.push(round);
    }
    reorder(&mut rounds);

    let Some(lines) = lines(rounds.clone()) else {
        // TODO log error
        return None;
    };

    // NOTE: given a number of players, the number of the matches is know
    // Then I can deal with an array of fixed size for the matches. It's not
    // like switching from Vec to array would hurt me, now would it?

    // TODO finish this code before next job maybe

    let mut ui_primitives: Vec<BracketPrimitives> = vec![];

    for (round, round_line) in rounds.clone().into_iter().zip(lines) {
        ui_primitives.push(BracketPrimitives::Match(round));
        let (left_col, right_col) = round_line.split_at(round_line.len() / 2);
        ui_primitives.push(BracketPrimitives::Block(left_col.to_vec()));
        ui_primitives.push(BracketPrimitives::Block(right_col.to_vec()));
    }
    if !rounds.is_empty() {
        ui_primitives.push(BracketPrimitives::Match(
            rounds.into_iter().last().expect("element"),
        ));
    }
    let columns = ui_primitives.len();

    cx.render(rsx!(div {
        MatchEditModal { isHidden: isMatchEditModalHidden }
        div {
            class: "grid grid-rows-1 grid-cols-{columns} flex",
            for s in ui_primitives.into_iter() {
                match s {
                    BracketPrimitives::Match(round) => rsx! { Round(cx, round) },
                    BracketPrimitives::Block(line) => rsx! {
                        ConnectMatchesBetweenRounds(line)
                    },
                }
            }
        }
    }))
}
