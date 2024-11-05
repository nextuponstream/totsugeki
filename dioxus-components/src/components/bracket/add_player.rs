//! Add player

#![allow(non_snake_case)]
use crate::components::SUBMIT_CLASS;
use crate::tournaments::Tournament;
use dioxus::prelude::*;
use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::format::Format;
use totsugeki::player::Player;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::validation::AutomaticMatchValidationMode;

/// Form to add player to the bracket
pub fn Form(cx: Scope) -> Element {
    let Some(tournament) = use_shared_state::<Tournament>(cx) else {
        log::error!("no tournament");
        return None;
    };
    let Some(single_elimination_bracket) = use_shared_state::<SingleEliminationBracket>(cx) else {
        log::error!("no single elimination bracket");
        return None;
    };
    let Some(double_elimination_bracket) = use_shared_state::<DoubleEliminationBracket>(cx) else {
        log::error!("no double elimination bracket");
        return None;
    };

    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Add new player"
        }

        form {
            prevent_default: "submit",
            onsubmit: move |event| {
                // println!("submitted {event:?}")
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

// TODO make it work for both single and double elimination bracket
/// Update stored bracket with new player using `Form`
#[allow(unused)]
fn add_player(
    tournament: &UseSharedState<Tournament>,
    single_elimination_bracket: &UseSharedState<SingleEliminationBracket>,
    double_elimination_bracket: &UseSharedState<DoubleEliminationBracket>,
    e: Event<FormData>,
) {
    log::trace!("Adding player...");
    log::debug!("{e:?}");
    log::debug!("{:?}", e.values);
    log::debug!("{:?}", e.values.get("name"));
    let Some(name) = e.values.get("name") else {
        return;
    };
    let name = if name.is_empty() {
        let i = tournament.read().get_participants().len() + 1;
        format!("player {}", i)
    } else {
        name.to_string()
    };
    let mut t = tournament.read().clone();
    let participants = t.get_participants();
    let player = Player::new(name);
    let participants = participants
        .add_participant(player.clone())
        .expect("should add participant");
    t.set_participants(participants);

    match t.get_format() {
        Format::SingleEliminationBracket => {
            let s = single_elimination_bracket.read().clone();
            let mut seeding = s.get_seeding().get();
            seeding.push(player.get_id());
            let s = SingleEliminationBracket::create(
                Seeding::new(seeding)
                    .expect("should set new seeding for single elimination bracket"),
                true,
            );
            *single_elimination_bracket.write() = s;
        }
        Format::DoubleEliminationBracket => {
            let d = double_elimination_bracket.read().clone();
            let mut seeding = d.get_seeding().get();
            seeding.push(player.get_id());
            let d = DoubleEliminationBracket::create(
                Seeding::new(seeding)
                    .expect("should set new seeding for double elimination bracket"),
                AutomaticMatchValidationMode::Flexible,
            );
            *double_elimination_bracket.write() = d;
        }
    }

    *tournament.write() = t;
    log::trace!("Added player");
}
