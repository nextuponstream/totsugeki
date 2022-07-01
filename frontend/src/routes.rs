use crate::common::tournament_server::Props;
use crate::pages::bracket::view::BracketViewCore;
use crate::pages::home::Home;
use yew::html;
use yew::prelude::*;
use yew_router::Routable;

#[derive(Debug, Clone, Copy, PartialEq, Routable)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/brackets")]
    Brackets,
    #[at("/404")]
    #[not_found]
    NotFound,
}

#[allow(clippy::trivially_copy_pass_by_ref)]
// ignored because switch requires this signature
pub fn switch(routes: &Route) -> Html {
    let props = Props::default();
    match routes {
        Route::Home => html! { <Home /> },
        Route::Brackets => html! { <BracketViewCore {props}/> },
        Route::NotFound => html! { <h1>{ "404" }</h1> },
    }
}
