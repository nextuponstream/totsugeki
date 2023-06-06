use dioxus::prelude::*;
use dioxus_components::components::{
    bracket::add_player::Form as AddPlayerForm,
    bracket::visualizer::{GeneralDetails, UpdateBracketDetails, View},
    navigation::NavBar,
};
use dioxus_components::Modal;
// use dioxus_components::{test, NavBar};
use totsugeki::bracket::Bracket;

fn main() {
    // init debug tool for WebAssembly
    wasm_logger::init(wasm_logger::Config::new(log::Level::Trace));
    console_error_panic_hook::set_once();

    dioxus_web::launch(app);
}

fn app(cx: Scope) -> Element {
    log::info!("Application started...");
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
