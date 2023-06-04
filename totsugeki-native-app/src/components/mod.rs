//! Components of the user interface

#![allow(non_snake_case)]
use dioxus::prelude::*;

pub mod bracket;
pub mod navigation;

/// submit button styling
pub(crate) const SUBMIT_CLASS: &str = "text-white bg-blue-700 hover:bg-blue-800 \
                        focus:ring-4 focus:ring-blue-300 font-medium \
                        rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 \
                        dark:bg-blue-600 dark:hover:bg-blue-700 \
                        focus:outline-none dark:focus:ring-blue-800";

/// Generic submission button with styling
pub(crate) fn Submit(cx: Scope) -> Element {
    cx.render(rsx!(
        div {
            input {
                class: "{SUBMIT_CLASS}",
                r#type: "submit",
            },
        }
    ))
}
