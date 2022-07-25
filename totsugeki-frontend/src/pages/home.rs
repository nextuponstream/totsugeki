use yew::Context;
use yew::{html, Component};
use yew_icons::{Icon, IconId};

pub struct Home;

impl Component for Home {
    type Message = ();
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, _ctx: &Context<Self>) -> yew::Html {
        html! {
            <>
                <section class="hero is-small">
                  <div class="hero-body" style="text-align:center">
                    <p class="title">{ "Totsugeki" }</p>
                    <p class="subtitle">{ "For TO's and players" }</p>
                    <div class="block">
                        <p>{ "Live the tournament life while only using discord in three commands or less!" }</p>
                    </div>
                    // TODO remove WIP status when advertised discord features are implemented
                    <div class="block">
                        <span class="icon-text">
                          <span class="icon has-text-warning">
                            <Icon icon_id={IconId::FontAwesomeSolidTriangleExclamation} />
                          </span>
                          <div style="text-align-center">{ "This project is WIP" }</div>
                          <span class="icon has-text-warning">
                            <Icon icon_id={IconId::FontAwesomeSolidTriangleExclamation} />
                          </span>
                        </span>
                    </div>
                  </div>
                </section>
                <section>
                    <div class="container is-max-desktop">
                        <div class="columns">
                            <div class="column">
                                <div class="box discord">
                                    <div class="content">
                                        <h2>{ "Players" }</h2>
                                        <p>{ "!join" }</p>
                                        <p>{ "!result 2-0" }</p>
                                        <p>{ "!nextmatch" }</p>
                                    </div>
                                </div>
                            </div>
                            <div class="column">
                                <div class="box discord">
                                    <div class="content">
                                        <h2>{ "TO's" }</h2>
                                        <p>{ "!bracket create my-monthly" }</p>
                                        <p>{ "!validatematch 123456" }</p>
                                        <p>{ "!finalize" }</p>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="content">
                            <h1 style="text-align:center">{ "What's more?" }</h1>
                        </div>
                        <div class="columns">
                            <div class="column">
                                <div class="content" style="text-align:center">
                                    { "This website! Checkout brackets tab!" }
                                </div>
                            </div>
                        </div>
                    </div>
                </section>
            </>
        }
    }
}
