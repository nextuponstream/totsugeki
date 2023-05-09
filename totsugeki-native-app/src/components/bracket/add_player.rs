//! Ajidqwoijdoqwj

#![allow(non_snake_case)]
use crate::components::Submit;
use dioxus::prelude::*;
use totsugeki::bracket::Bracket;

pub fn Form(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Add new player"
        }

        form {
            onsubmit: move |event| { add_player(bracket, event ) },

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

            Submit {}
        }
    ))
}

fn add_player(bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    let name = e.values.get("name").expect("name");
    let name = if name.is_empty() {
        let i = bracket.read().get_participants().len() + 1;
        format!("player {}", i)
    } else {
        name.to_string()
    };
    let b = match bracket.write().clone().add_participant(&name) {
        Ok(b) => b,
        Err(e) => {
            println!("{e}"); // TODO use a logging library
            return;
        }
    };

    *bracket.write() = b;
}
