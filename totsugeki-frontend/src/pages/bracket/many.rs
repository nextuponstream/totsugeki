//! Page displaying many brackets

use crate::common::api::Props;
use crate::{get_client, routes::Route};
use totsugeki::bracket::Bracket;
use totsugeki_api_request::{bracket::fetch, RequestError};
use web_sys::HtmlInputElement;
use yew::prelude::*;
use yew::{html, Component, Context, Html};
use yew_router::prelude::*;

use super::FetchState;

/// View of many brackets
pub struct View {
    /// state of the page fetch request to display a bracket list
    fetch_state: FetchState<Vec<Bracket>>,
    /// html nodes over the filter input and search button
    refs: Vec<NodeRef>,
    /// indicate if a user facing error message should be displayed for the offset input
    input_error_offset: bool,
}

/// Update bracket view
#[derive(Debug)]
pub enum Msg {
    /// Update UI with bracket list
    GetBrackets,
    /// Update view after API call to fetch bracket
    SetBracketsFetchState(FetchState<Vec<Bracket>>),
}

impl Component for View {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        // immediately fetch some brackets to avoid empty page
        ctx.link().send_message(Msg::GetBrackets);
        Self {
            fetch_state: FetchState::NotFetching,
            refs: vec![NodeRef::default(), NodeRef::default()],
            input_error_offset: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetBrackets => {
                let bracket_name_input = &self.refs[0];
                let bracket_name_value = bracket_name_input
                    .cast::<HtmlInputElement>()
                    .expect("Bracket name field missing")
                    .value();
                let offset_input = &self.refs[1];
                let offset_value = offset_input
                    .cast::<HtmlInputElement>()
                    .expect("Offset field missing")
                    .value();
                let offset: i64 = match offset_value.parse() {
                    Ok(i) => {
                        self.input_error_offset = false;
                        i
                    }
                    Err(_e) => {
                        // placeholder is not an error
                        if !offset_value.is_empty() {
                            self.input_error_offset = true;
                        }
                        0
                    }
                };
                let bracket_name_filter = if bracket_name_value.is_empty() {
                    None
                } else {
                    Some(bracket_name_value)
                };
                let addr = ctx.props().props.get_backend_addr();
                ctx.link().send_future(async move {
                    match fetch(get_client(), addr.as_str(), bracket_name_filter, offset).await {
                        Ok(brackets) => Msg::SetBracketsFetchState(FetchState::Success(brackets)),
                        Err(e) => Msg::SetBracketsFetchState(FetchState::Failed(e)),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetBracketsFetchState(FetchState::Fetching));
                false
            }
            Msg::SetBracketsFetchState(fetch_state) => {
                self.fetch_state = fetch_state;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
            { filter_brackets(self, ctx) }
            {
        match &self.fetch_state {
            FetchState::NotFetching => html! { "" },
            FetchState::Fetching => html! {
                <progress class="progress is-medium is-dark" max="100" />
            },
            FetchState::Success(brackets) => html! {
                <div class="content">
                    <table>
                        <thead>
                          <tr>
                            <th>{ "ID" }</th>
                            <th>{ "Name" }</th>
                          </tr>
                        </thead>
                         <tbody>{
                             brackets.iter().map(|b| html! {
                                 <tr>
                                     <td>
                                        <Link<Route>
                                            to={Route::Bracket { bracket_id: b.get_id() }}>
                                            {b.get_id()}
                                        </Link<Route>>
                                    </td>
                                    <td>{b.get_bracket_name()}</td>
                                 </tr>
                             }).collect::<Html>()
                         }</tbody>
                    </table>
                </div>
            },
            FetchState::Failed(err) => match err {
                    RequestError::Request(_e, msg)=> html!{ format!("An error has happened: {}",msg) },
                    RequestError::BracketParsingError(e) =>  html! { format!("{e}") },
                    RequestError::MatchIdParsingError(e) =>  html! { format!("{e}") },
                },
            }
            }
        </>
        }
    }
}

/// Html view of bracket list
fn filter_brackets(view_self: &View, ctx: &Context<View>) -> Html {
    html! {
        // NOTE: prevent_default prevents form from reopening the same page because of submit
        // button in a form
        <form onsubmit={ctx.link().callback(|e: FocusEvent| {e.prevent_default(); Msg::GetBrackets })}>
        <div class="columns is-centered">
            <div class="column is-offset-one-quarter">
                <div class="field is-horizontal">
                    <div class="field-label is-normal">
                        <label class="label">{ "Name" }</label>
                    </div>
                    <div class="field-body">
                        <div class="field">
                            <p class="control">
                                <input class="input" ref={view_self.refs[0].clone()}
                                type="text" placeholder="name (exact match)" />
                            </p>
                        </div>
                    </div>
                </div>
            </div>
            <div class="column">
                <div class="field is-horizontal">
                    <div class="field-label is-normal">
                        <label class="label">{ "Offset" }</label>
                    </div>
                    <div class="field-body">
                        <div class="field">
                            <p class="control">
                                // TODO replace offset by pagination
                                <input class={classes!("input",
                                if view_self.input_error_offset { "is-danger" } else { "" })}
                                ref={view_self.refs[1].clone()}
                                type="text" placeholder="0" />
                            </p>
                            if view_self.input_error_offset { <p class="help is-danger">{ "Could not parse number" }</p> }
                        </div>
                    </div>
                </div>
            </div>
            <div class="column is-narrow">
                <div class="field is-horizontal">
                    <div class="field-label" />
                    <div class="field-body">
                        <div class="field">
                            <p class="control">
                                // NOTE: prevent_default prevents button from submit twice
                                // dev tool (F12) > Network tab > click button
                                <input class="button is-link" type="submit"
                                    onclick={ctx.link().callback(|e: MouseEvent| {e.prevent_default(); Msg::GetBrackets})}
                                    value="get brackets"
                                />
                            </p>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        </form>
    }
}
