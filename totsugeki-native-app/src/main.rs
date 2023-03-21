#![allow(non_snake_case)]
// import the prelude to get access to the `rsx!` macro and the `Scope` and `Element` types
use chrono::prelude::*;
use dioxus::prelude::*;
use dioxus_desktop::Config;
use totsugeki::bracket::Bracket;
use totsugeki::matches::Id as MatchId;
use totsugeki_native_app::components::{
    bracket::visualizer::{AddPlayerForm, GeneralDetails, UpdateBracketDetails, View},
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
    let b = Bracket::new(
        "test",
        totsugeki::format::Format::SingleElimination,
        totsugeki::seeding::Method::Strict,
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        true,
    );
    use_shared_state_provider(cx, || b); // TODO revert to Bracket::default
    use_shared_state_provider::<Option<MatchId>>(cx, || None);

    cx.render(rsx! {
        style { include_str!("../resources/tailwind.css") }

        div {
            class: "pl-2",

            Navigation {}
            UpdateBracketDetails {}
            AddPlayerForm {}
            View {}
            GeneralDetails {}
        }

    })
}
