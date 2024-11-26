//! Ajidqwoijdoqwj

#![allow(non_snake_case)]
use crate::components::SUBMIT_CLASS;
use crate::tournaments::Tournament;
use dioxus::prelude::*;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::format::Format;
use totsugeki::matches::result::MatchFormat;
use totsugeki::player::Player;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::validation::AutomaticMatchValidationMode;

/// Form to add player to the bracket
pub fn Form(cx: Scope) -> Element {
    let single_elimination_bracket =
        use_shared_state::<SingleEliminationBracket>(cx).expect("bracket");
    let double_elimination_bracket =
        use_shared_state::<DoubleEliminationBracket>(cx).expect("bracket");
    let tournament = use_shared_state::<Tournament>(cx).expect("tournament");

    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Add new player"
        }

        form {
            onsubmit: move |event| {
                add_player(tournament, single_elimination_bracket, double_elimination_bracket, event);
            },

            div {
                class: "pb-2",
                label { "Player name" }
                input {
                    class: "border border-gray-300 text-sm rounded-lg \
                            focus:ring-blue-500 block p-2.5 \
                            focus:border-blue-500",
                    name: "name",
                }
            }

            input {
                class: "{SUBMIT_CLASS}",
                r#type: "submit",
            },
        }
    ))
}

/// Update stored bracket with new player using `Form`
fn add_player(
    tournament: &UseSharedState<Tournament>,
    single_elimination_bracket: &UseSharedState<SingleEliminationBracket>,
    double_elimination_bracket: &UseSharedState<DoubleEliminationBracket>,
    e: Event<FormData>,
) {
    let Some(name) = e.values.get("name") else {
        return;
    };
    let Some(name) = name.first() else { return };
    let name = if name.is_empty() {
        let i = tournament.read().get_participants().len() + 1;
        format!("player {}", i)
    } else {
        name.to_string()
    };
    let t = tournament.read().clone();
    let participants = t.get_participants();
    let participants = match participants.add_participant(Player::new(name)) {
        Ok(b) => b,
        Err(e) => {
            println!("{e}"); // TODO use a logging library
            return;
        }
    };
    let mut t_update = Tournament::default();
    t_update.set_participants(participants);
    *tournament.write() = t_update;
    match t.format {
        Format::SingleEliminationBracket => {
            *single_elimination_bracket.write() = SingleEliminationBracket::create(
                t.get_participants().get_seeding(),
                true,
                MatchFormat::ft3(), // FIXME should be user provided
                None,               // FIXME should be user provided
            );
        }
        Format::DoubleEliminationBracket => {
            *double_elimination_bracket.write() = DoubleEliminationBracket::create(
                t.get_participants().get_seeding(),
                AutomaticMatchValidationMode::Flexible,
                MatchFormat::ft3(), // FIXME should be user provided
                None,               // FIXME should be user provided
            );
        }
    }
}
