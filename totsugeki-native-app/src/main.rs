#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use chrono::prelude::*;
use dioxus::prelude::*;
use dioxus_desktop::Config;
use totsugeki::bracket::Bracket;
use totsugeki_native_app::components::{
    bracket::add_player::Form as AddPlayerForm,
    bracket::visualizer::{GeneralDetails, UpdateBracketDetails, View},
    navigation::Navigation,
};
use totsugeki_native_app::Modal;

fn main() {
    hot_reload_init!();
    // TODO add some auto-completion for css class
    // wait for https://github.com/helix-editor/helix/issues/2213
    // TODO i18n (fluent crate)
    dioxus_desktop::launch_cfg(App, Config::new());
}

fn App(cx: Scope) -> Element {
    dioxus_desktop::use_window(cx).set_title("Totsugeki bracket viewer");
    let b = Bracket::new(
        "test",
        totsugeki::format::Format::DoubleElimination,
        totsugeki::seeding::Method::Strict,
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        true,
    );
    use_shared_state_provider(cx, || b); // TODO revert to Bracket::default
    use_shared_state_provider::<Option<Modal>>(cx, || None);

    cx.render(rsx! {
        style { include_str!("../resources/tailwind.css") }

        div {
            class: "pl-2",

            Navigation {}
            div {
                class: "pt-2 flex flex-row justify-around",
                UpdateBracketDetails {}
                AddPlayerForm {}
            }
            View {}
            GeneralDetails {}
        }

    })
}
