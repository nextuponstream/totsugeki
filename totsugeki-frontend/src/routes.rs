//! Router

use crate::common::api::Props;
use crate::pages::bracket::many::View as BracketListView;
use crate::pages::bracket::single::View as SingleBracketView;
use crate::pages::home::Home;
use totsugeki::bracket::Id as BracketId;
use yew::html;
use yew::prelude::*;
use yew_router::Routable;

/// Router to other pages
#[derive(Debug, Clone, Copy, Eq, PartialEq, Routable)]
pub enum Route {
    /// Home page
    #[at("/")]
    Home,
    /// Bracket list page
    #[at("/brackets")]
    Brackets,
    /// View over single bracket
    #[at("/bracket/:bracket_id")]
    Bracket {
        /// Bracket id
        bracket_id: BracketId,
    },
    /// Unknown page
    #[at("/404")]
    #[not_found]
    NotFound,
}

/// Router function
#[allow(clippy::trivially_copy_pass_by_ref)]
// ignored because switch requires this signature
#[allow(clippy::let_unit_value)]
pub fn switch(routes: &Route) -> Html {
    let props = Props::default();
    match routes {
        Route::Home => html! { <Home /> },
        Route::Brackets => html! { <BracketListView {props}/> },
        Route::Bracket { bracket_id } => {
            html! { <SingleBracketView bracket_id={*bracket_id} api={props}/> }
        }
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}
