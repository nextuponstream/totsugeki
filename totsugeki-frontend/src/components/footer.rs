//! Footer component

use yew::prelude::*;
use yew_icons::{Icon, IconId};

/// Footer web component
pub struct Footer;

impl Component for Footer {
    type Message = ();

    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self {}
    }

    fn view(&self, _ctx: &Context<Self>) -> Html {
        html! {
            <footer>
                <div class="columns is-multiline is-mobile" style="padding:1em;">
                    <div class="column is-one-third">
                        <p style="text-align:center"><i>{ "\"Roundstart goldburst is a good strategy\"" }</i></p>
                        <div style="text-align:right"><p>{ "-Sun Tzu, probably" }</p></div>
                    </div>
                    <div class="column" style="text-align:center">
                        <strong>{ "Totsugeki" }</strong>{ " by " }
                        <a href="https://github.com/nextuponstream" >{ "nextuponstream" }</a>
                    </div>
                    <div class="column is-one-third" style="text-align:right;padding-right:3em">
                        <span class="icon-text">
                            <span>{ "Totsugeki code" }</span>
                            <span class="icon">
                                // black otherwise inherit "blue link" color for logo
                                <a href="https://github.com/nextuponstream/totsugeki" style="color:black;">
                                    <Icon icon_id={IconId::BootstrapGithub} />
                                </a>
                            </span>
                        </span>
                    </div>
                </div>
            </footer>
        }
    }
}
