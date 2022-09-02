//! View of a single bracket

use crate::common::api::Api;
use crate::get_client;
use totsugeki::bracket::Id as BracketId;
use totsugeki::bracket::Raw;
use totsugeki::matches::print_player_name;
use totsugeki_api_request::{bracket::get_from_id, RequestError};
use yew::prelude::*;
use yew::{Component, Properties};

use super::FetchState;

/// Bracket page properties
#[derive(Eq, PartialEq, Properties, Clone)]
pub struct Props {
    /// Bracket id
    pub bracket_id: BracketId,
    /// Fetch data from api
    pub api: Api,
}

/// View over a single bracket
pub struct View {
    /// state of the page fetch request to display a bracket
    fetch_state: FetchState<Raw>,
}

/// Update bracket view
pub enum Msg {
    /// Update page with bracket
    GetBracket,
    /// Update view after API call to fetch bracket
    SetBracketFetchState(FetchState<Raw>),
}

impl Component for View {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &yew::Context<Self>) -> Self {
        ctx.link().send_message(Msg::GetBracket);
        Self {
            fetch_state: FetchState::NotFetching,
        }
    }

    fn update(&mut self, ctx: &yew::Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::GetBracket => {
                let addr = ctx.props().api.get_backend_addr();
                let bracket_id = ctx.props().bracket_id;
                ctx.link().send_future(async move {
                    match get_from_id(get_client(), addr.as_str(), bracket_id).await {
                        Ok(bracket) => Msg::SetBracketFetchState(FetchState::Success(bracket)),
                        Err(e) => Msg::SetBracketFetchState(FetchState::Failed(e)),
                    }
                });
                ctx.link()
                    .send_message(Msg::SetBracketFetchState(FetchState::Fetching));
                false
            }
            Msg::SetBracketFetchState(state) => {
                self.fetch_state = state;
                true
            }
        }
    }

    fn view(&self, _ctx: &yew::Context<Self>) -> yew::Html {
        match &self.fetch_state {
            FetchState::NotFetching => html! { "" },
            FetchState::Fetching => html! {
                <progress class="progress is-medium is-dark" max="100" />
            },
            FetchState::Success(bracket) => {
                let players = bracket.get_players_list();
                let players = players.iter().map(|p| {
                    html! { <p>{ p.get_name() }</p> }
                });
                let matches = bracket.matches.iter().map(|m| {
                    let player_1 =
                        print_player_name(m.get_players()[0], &bracket.get_players_list())
                            .unwrap_or_else(|| "ERROR".to_string());
                    let player_2 =
                        print_player_name(m.get_players()[1], &bracket.get_players_list())
                            .unwrap_or_else(|| "ERROR".to_string());
                    html! { <p>{ player_1 }<b>{ " VS " }</b>{ player_2 } </p> }
                });
                html! {
                    <div class="content">
                        <p>{ "Bracket: " } {bracket.bracket_name.clone()} {" ("} {bracket.bracket_id} {")"}</p>
                        <p>{ "Start time: " } {bracket.start_time.to_string()}</p>
                        <p>{ "Format: " } {bracket.format.to_string()}</p>
                        <p>{ "Seeding type: " } {bracket.seeding_method.to_string()}</p>
                        <p>{ "Players:"}</p>
                        { for players }
                        <p>{ "Matches:"}</p>
                        { for matches }
                    </div>
                }
            }
            FetchState::Failed(err) => match err {
                RequestError::Request(_, msg) => {
                    html! { format!("An error has happened: {}", msg) }
                }
                RequestError::BracketParsingError(e) => html! { e.to_string() },
                RequestError::MatchIdParsingError(e) => html! { e.to_string() },
                RequestError::NextMatch(e) => html! { e.to_string() },
                RequestError::PlayerParsingError(e) => html! { e.to_string() },
            },
        }
    }
}
