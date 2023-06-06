//! Add player

#![allow(non_snake_case)]
use crate::components::SUBMIT_CLASS;
use dioxus::prelude::*;
use totsugeki::bracket::Bracket;

/// Form to add player to the bracket
pub fn Form(cx: Scope) -> Element {
    let Some(bracket) = use_shared_state::<Bracket>(cx) else {
        log::error!("no bracket");
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
                add_player(bracket, event);
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
fn add_player(bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    log::trace!("Adding player...");
    log::debug!("{e:?}");
    log::debug!("{:?}", e.values);
    log::debug!("{:?}", e.values.get("name"));
    let Some(name) = e.values.get("name") else {return};
    // let Some(name) = name.first() else {return};
    let name = if name.is_empty() {
        let i = bracket.read().get_participants().len() + 1;
        format!("player {}", i)
    } else {
        name.to_string()
    };
    let b = match bracket.write().clone().add_participant(&name) {
        Ok(b) => b,
        Err(e) => {
            log::error!("{e}");
            return;
        }
    };

    *bracket.write() = b;
    log::trace!("Added player");
}
