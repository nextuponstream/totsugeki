//! Navigation bar component

use crate::routes::Route;
use yew::prelude::*;
use yew::{html, Component};
use yew_router::prelude::*;

/// Update navigation bar component
pub enum Msg {
    /// Toggle navigation bar on small screen
    ToggleNavbar,
}

/// Navigation bar component
pub struct Navbar {
    /// Visibility of the navigation bar
    navbar_active: bool,
}

impl Component for Navbar {
    type Message = Msg;

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            navbar_active: false,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleNavbar => {
                self.navbar_active = !self.navbar_active;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let Self { navbar_active, .. } = *self;
        let active_class = if navbar_active { "is-active" } else { "" };

        html! {
            <nav class="navbar" role="navigation" aria-label="main navigation">
                <div class="navbar-brand">
                    <div class="navbar-item" style="width:130px;height:100px">
                        <img src="/GGST_May_Mr._Dolphin_Horizontal-6.png" style="min-width:100%;min-height:100%" />
                    </div>
                    <h1 class="navbar-item is-size-3">{ "Totsugeki" }</h1>
                    <button class={classes!("navbar-burger", "burger", active_class)}
                        aria-label="menu" aria-expanded="false"
                        onclick={ctx.link().callback(|_| Msg::ToggleNavbar)}
                    >
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                        <span aria-hidden="true"></span>
                    </button>
                </div>
                <div class={classes!("navbar-menu", active_class)}>
                    <div class="navbar-start">
                        <Link<Route> classes={classes!("navbar-item")} to={Route::Home}>
                            { "Home" }
                        </Link<Route>>
                      <Link<Route> classes="navbar-item" to={Route::Brackets}>
                        { "Brackets" }
                      </Link<Route>>
                    </div>
                </div>
                //<div class="navbar-end">
                //  <div class="navbar-item">
                //    <div class="buttons">
                //      // TODO Add login
                //      <a class="button is-primary"><strong>{ "Sign up" }</strong></a>
                //      <a class="button is-light">{ "Log in" }</a>
                //    </div>
                //  </div>
                //</div>
            </nav>
        }
    }
}
