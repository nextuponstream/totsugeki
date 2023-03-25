//! Ajidqwoijdoqwj

#![allow(non_snake_case)]
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
                    class: "border border-gray-300 text-gray-900 text-sm \
                            rounded-lg focus:ring-blue-500 block p-2.5 \
                            focus:border-blue-500",
                    name: "name",
                }
            }

            // TODO refactor submission button in reusable component submit button
            div {
                input {
                    class: "text-white bg-blue-700 hover:bg-blue-800 \
                            focus:ring-4 focus:ring-blue-300 font-medium \
                            rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 \
                            dark:bg-blue-600 dark:hover:bg-blue-700 \
                            focus:outline-none dark:focus:ring-blue-800",
                    r#type: "submit",
                },
            }

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
