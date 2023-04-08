#![allow(non_snake_case)]

use crate::{
    components::bracket::displayable_round::Round,
    components::bracket::match_edit::MatchEditModal,
    components::Submit,
    single_elimination::{ordering::reorder_rounds, partition::winner_bracket},
    DisplayableMatch, Modal,
};
use chrono::{TimeZone, Utc};
use dioxus::prelude::*;
use totsugeki::{bracket::Bracket, format::Format, matches::Id as MatchId};

use super::displayable_round::{winner_bracket_lines, BoxWithBorder};

pub fn GeneralDetails(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    let details = bracket.read().to_string();
    let format = bracket.read().get_format().to_string();
    let participants = bracket.read().get_participants();
    let n = participants.len();

    cx.render(rsx!(div {
        h1 {
            class: "text-lg",
            "General details"
        }
        p { details }
        p {
            label { class: "pr-2", "Format:" }
            format
        }
        p {
            label { class: "pr-2", "Players:" }
            n.to_string()
        }
        for p in participants.get_players_list().iter() {
            p { p.to_string() }
        }
    }))
}

pub fn UpdateBracketDetails(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    cx.render(rsx!(

        h2 {
            class: "text-lg",
            "Update bracket"
        }
        form {
            onsubmit: move |event| { update_bracket(bracket, event ) },

            div {
                class: "pb-2",
               label { "Name" }
                input {
                    class: "border border-gray-300 text-gray-900 text-sm \
                            rounded-lg focus:ring-blue-500 block p-2.5 \
                            focus:border-blue-500",
                    name: "name",
                }
            }

            div {
                class: "pb-2",
                label {
                    class: "pr-2",
                    "Format"
                }
                select {
                    name: "format",
                    option { "single-elimination" }
                    option { "double-elimination" }
                }
            }

            Submit {}
        }
    ))
}

fn update_bracket(bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    let name = e.values.get("name").expect("name");
    let format = e.values.get("format").expect("format");
    let is_valid = true;
    let (format, is_valid) = match format.parse::<Format>() {
        Ok(f) => (f, is_valid),
        Err(_e) => (Format::default(), false),
    };

    if !is_valid {
        return;
    }

    *bracket.write() = Bracket::new(
        name,
        format,
        totsugeki::seeding::Method::Strict,
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        true,
    );
}

pub fn View(cx: Scope) -> Element {
    let format = match use_shared_state::<Bracket>(cx) {
        Some(bracket_ref) => bracket_ref.read().get_format(),
        None => Bracket::default().get_format(),
    };
    let view = match format {
        Format::SingleElimination => SingleEliminationBracketView(cx),
        Format::DoubleElimination => DoubleEliminationBracketView(cx),
    };

    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Bracket view"
        }
        view

    ))
}

#[derive(PartialEq, Clone)]
struct SomeProps {
    id: &'static MatchId,
}

enum DisplayStuff {
    /// Display match
    Match(Vec<DisplayableMatch>),
    /// Padding block with
    Block(Vec<BoxWithBorder>),
}

fn SingleEliminationBracketView(cx: Scope) -> Element {
    let modal = use_shared_state::<Option<Modal>>(cx).expect("modal to show");
    let isMatchEditModalHidden = !matches!(*modal.read(), Some(Modal::EnterMatchResult(_, _, _)));
    let bracket = match use_shared_state::<Bracket>(cx) {
        Some(bracket_ref) => bracket_ref.read().clone(),
        None => Bracket::default(),
    };

    let matches = bracket.get_matches();
    let participants = bracket.get_participants();

    let mut rounds = winner_bracket(matches, &participants);
    reorder_rounds(&mut rounds);

    let lines = winner_bracket_lines(rounds.clone());

    // NOTE: given a number of players, the number of the matches is know
    // Then I can deal with an array of fixed size for the matches. It's not
    // like switching from Vec to array would hurt me, now would it?

    // TODO finish this code before next job maybe

    let mut stuff: Vec<DisplayStuff> = vec![];

    // TODO avoid copy (always split everything and last of rounds)
    for (round, round_line) in rounds.clone().into_iter().zip(lines) {
        stuff.push(DisplayStuff::Match(round));
        let (left_col, right_col) = round_line.split_at(round_line.len() / 2);
        stuff.push(DisplayStuff::Block(left_col.to_vec()));
        stuff.push(DisplayStuff::Block(right_col.to_vec()));
    }
    if !rounds.is_empty() {
        stuff.push(DisplayStuff::Match(rounds.into_iter().last().unwrap()));
    }
    let columns = stuff.len();

    cx.render(rsx!(div {
        MatchEditModal { isHidden: isMatchEditModalHidden }
        div {
            class: "grid grid-rows-1 grid-cols-{columns} flex",
            for s in stuff {
                match s {
                    DisplayStuff::Match(round) => rsx! { Round(cx, round) },
                    DisplayStuff::Block(line) => rsx! {
                        RoundWithLines(line)
                    },
                }
            }
        }
    }))
}

fn RoundWithLines<'a, 'b>(lines: Vec<BoxWithBorder>) -> LazyNodes<'a, 'b> {
    rsx!(
        div {
            class: "grid grid-cols-1",
            lines.iter().map(|b| {
                let left = if b.left { "border-l" } else { "" };
                let bottom = if b.bottom { "border-b" } else { "" };
                rsx!(div {
                    class: "{left} {bottom}",
                    // "yooo" // TODO remove
                })
            })
        }
    )
}

fn DoubleEliminationBracketView(cx: Scope) -> Element {
    // let n = bracket.get_participants().len();
    // TODO partition matches according to powers of two
    // let matches = bracket.get_matches();

    cx.render(rsx!(
        "" // p { n.to_string() }
           // for m in matches.iter() {
           // p { m.to_string() }
           // }
    ))
}
