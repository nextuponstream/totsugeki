//! App component (main container)

use crate::components::footer::Footer;
use crate::components::navbar::Navbar;
use crate::routes::{switch, Route};
use yew::prelude::*;
use yew::{html, Component};
use yew_router::prelude::*;

/// Main web container of frontend
pub struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn update(&mut self, _ctx: &Context<Self>, _msg: Self::Message) -> bool {
        false
    }

    #[allow(clippy::let_unit_value)]
    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <BrowserRouter>
                <header><Navbar /></header>
                <main>
                   <div class="main-content">
                            <Switch<Route> render ={Switch::render(switch)} />
                   </div>
                </main>
                <Footer />
            </BrowserRouter>
        }
    }
}
