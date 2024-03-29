#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use dioxus_desktop::Config;
use totsugeki::bracket::Bracket;
use totsugeki_native_app::components::{
    bracket::add_player::Form as AddPlayerForm,
    bracket::visualizer::{GeneralDetails, UpdateBracketDetails, View},
    navigation::NavBar,
};
use totsugeki_native_app::Modal;

fn main() {
    hot_reload_init!();
    // wasm_logger::init(wasm_logger::Config::default());
    // console_error_panic_hook::set_once();

    // TODO add some auto-completion for css class
    // wait for https://github.com/helix-editor/helix/issues/2213
    // TODO internationalisation i18n https://github.com/DioxusLabs/dioxus-std
    dioxus_desktop::launch_cfg(App, Config::new());
    // dioxus_web::launch(App);
}

fn App(cx: Scope) -> Element {
    dioxus_desktop::use_window(cx).set_title("Totsugeki bracket viewer");
    let b = Bracket::default();
    use_shared_state_provider(cx, || b);
    use_shared_state_provider::<Option<Modal>>(cx, || None);

    cx.render(rsx! {
        style { include_str!("../resources/tailwind.css") }

        div {
            class: "pl-2",

            NavBar {}
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
