//! View of a single bracket

use crate::common::api::Props;
use yew::Component;

/// View over a single bracket
pub struct View {}

/// Update bracket view
pub enum Msg {
    /// Update page with bracket
    GetBracket,
}

impl Component for View {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        todo!()
    }

    fn view(&self, ctx: &yew::Context<Self>) -> yew::Html {
        todo!()
    }
}
