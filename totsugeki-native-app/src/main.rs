#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use dioxus::prelude::*;
use dioxus_desktop::Config;
use totsugeki::bracket::Bracket;
use totsugeki_native_app::components::{
    bracket::visualizer::{GeneralDetails, UpdateBracketDetails},
    navigation::Navigation,
};

fn main() {
    hot_reload_init!();
    // TODO add some auto-completion for css class
    // wait for https://github.com/helix-editor/helix/issues/2213
    // TODO i18n (fluent crate)
    // TODO minify used css
    dioxus_desktop::launch_cfg(App, Config::new());
}

fn App(cx: Scope) -> Element {
    use_shared_state_provider(cx, Bracket::default);

    cx.render(rsx! {
        style { include_str!("../resources/tailwind.css") }

        div {
            class: "pl-2",

            Navigation {}
            UpdateBracketDetails {}
            GeneralDetails {}
        }

    })
}
