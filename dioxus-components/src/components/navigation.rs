//! Navigation bar of totsugeki app

#![allow(non_snake_case)]

use dioxus::prelude::*;

/// Navigation bar
pub fn NavBar(cx: Scope) -> Element {
    cx.render(rsx!(
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
    ))
}
