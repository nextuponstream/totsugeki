//! Frontend of Totsugeki

#![recursion_limit = "512"]
#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]

mod common;
mod components;
mod pages;
mod routes;

use components::app::App;
use wasm_bindgen::prelude::*;

/// Start (main) function for web app
#[wasm_bindgen]
pub fn run_app() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    yew::start_app::<App>();
    Ok(())
}

/// Get http client
#[must_use]
pub fn get_client() -> reqwest::Client {
    reqwest::Client::new()
}
