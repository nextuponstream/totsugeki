#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use chrono::{TimeZone, Utc};
use dioxus::prelude::Event;
use dioxus::prelude::*;
use dioxus_desktop::Config;
use totsugeki::bracket::Bracket;

fn main() {
    hot_reload_init!();
    // TODO add some auto-completion for css class
    // wait for https://github.com/helix-editor/helix/issues/2213
    // TODO i18n (fluent crate)
    // TODO minify used css
    dioxus_desktop::launch_cfg(App, Config::new());
}

fn App(cx: Scope) -> Element {
    let bracket = use_state(cx, Bracket::default);

    cx.render(rsx! {
        style { include_str!("../resources/tailwind.css") }
        div {
            class: "grid grid-cols-3 gap-4 p-4 bg-blue-100",
            div {
                class: "text-start p-2",
                format!("totsugeki-native-app v{}", env!("CARGO_PKG_VERSION")),
            }
            div {
                class: "text-center bg-blue-200 p-2",
                "Totsugeki!",
            }
            div {
                class: "text-end p-2",
                "Lang TODO",
            }
        }

        form {
            onsubmit: move |event| { update_bracket(bracket, event ) },
            class: "p-1",

            div {
                class: "pb-2",
                label { "Name" }
                input {
                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
                    name: "name",
                }
            }

            div {
                class: "",
                // TODO missing hover style
                input {
                    class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800",
                    r#type: "submit",
                },
            }

        }


        div {
            h1 { "Bracket" }
            div {
                bracket.to_string()
            }
        }
    })
}

fn update_bracket(bracket: &UseState<Bracket>, e: Event<FormData>) {
    println!("{e:?}");
    let name = e.values.get("name").expect("name");

    bracket.set(Bracket::new(
        name,
        totsugeki::format::Format::DoubleElimination,
        totsugeki::seeding::Method::Strict,
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        true,
    ));

    println!("{bracket}");
}
