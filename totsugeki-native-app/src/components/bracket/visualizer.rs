#![allow(non_snake_case)]

use chrono::{TimeZone, Utc};
use dioxus::prelude::*;
use totsugeki::{
    bracket::Bracket, 
    matches::Id as MatchId,
    format::Format, opponent::Opponent
};
use crate::{
    single_elimination::ordering::reorder_rounds, 
    single_elimination::partition::winner_bracket, 
    DisplayableMatch
};

pub fn GeneralDetails(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    let details = bracket.read().to_string();
    let format = bracket.read().get_format().to_string();
    let participants = bracket.read().get_participants();
    let n = participants.len();

    cx.render(rsx!(div {
        h1 {
            class: "text-lg",
            "General details"
        }
        p { details }
        p { 
            label { class: "pr-2", "Format:" }
            format
        }
        p { 
            label { class: "pr-2", "Players:" }
            n.to_string()
        }
        for p in participants.get_players_list().iter() {
            p { p.to_string() }
        }
    }))
}

pub fn UpdateBracketDetails(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    cx.render(rsx!(
        
        h2 { 
            class: "text-lg",
            "Update bracket"
        }
        form {
            onsubmit: move |event| { update_bracket(bracket, event ) },

            div {
                class: "pb-2",
                label { "Name" }
                input {
                    class: "border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5",
                    name: "name",
                }
            }
            
            div {
                class: "pb-2",
                label { 
                    class: "pr-2",
                    "Format"
                }
                select {
                    name: "format",
                    option { "single-elimination" }
                    option { "double-elimination" }
                }
            }

            div {
                input {
                    class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800",
                    r#type: "submit",
                },
            }

        }
        
        MatchEdit {}
    ))
}

fn update_bracket(bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    let name = e.values.get("name").expect("name");
    let format = e.values.get("format").expect("format");
    let is_valid = true;
    let (format, is_valid) = match format.parse::<Format>() {
        Ok(f) => (f, is_valid),
        Err(_e) => (Format::default(), false),
    };

    if !is_valid {
        return;
    }

    *bracket.write() = Bracket::new(
        name,
        format,
        totsugeki::seeding::Method::Strict,
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        true,
    );
}

fn update_result(cx: Scope, bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    let Some(m_id) = *use_shared_state::<Option<MatchId>>(cx).expect("match id").read() else {
        return;
    };
    
    let b = bracket.write().clone();
    let matches = b.get_matches();
    
    let Some(m) = matches.iter().find(|m| m.get_id() == m_id) else {
        return;
    };
    
    let (p1, p2) = match m.get_players() {
        [Opponent::Player(p1), Opponent::Player(p2)] => (p1, p2),
        _ => return,
    };
    let r1 = e.values.get("result_1").expect("result for p1");
    let r1 = r1.parse::<i8>().unwrap();
    let r2 = e.values.get("result_2").expect("result for p2");
    let r2 = r2.parse::<i8>().unwrap();
    let result = (r1,r2);
    
    *bracket.write() = b.tournament_organiser_reports_result(p1, result, p2).expect("new bracket").0;
}

pub fn MatchEdit(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");
    let m_id = match use_shared_state::<Option<MatchId>>(cx) {
        Some(r) => match *r.read(){
            Some(id) => id.to_string(),
            None => "".to_string(),
        }, 
        _ => "".to_string(),
    };

    cx.render(rsx!(div {
        h2 {
            class: "text-lg",
            "Update match result"
        }
        form {
            onsubmit: move |event| { update_result(cx, bracket, event) },

            div { "Match ID: {m_id}" }
            div {
                class: "pb-2",
                label { "Result for player 1" }
                input {
                    class: "border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5",
                    name: "result_1",
                }
            }
           
            div {
                class: "pb-2",
                label { "Result for player 2" }
                input {
                    class: "border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5",
                    name: "result_2",
                }
            }
           
            // TODO refactor submission button in reusable component submit button
            div {
                input {
                    class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800",
                    r#type: "submit",
                },
            }

        }
    }))
}

pub fn AddPlayerForm(cx: Scope) -> Element {
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");

    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Add new player"
        }

        form {
            onsubmit: move |event| { add_player(bracket, event ) },

            div {
                class: "pb-2",
                label { "Player name" }
                input {
                    class: "border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5",
                    name: "name",
                }
            }
           
            // TODO refactor submission button in reusable component submit button
            div {
                input {
                    class: "text-white bg-blue-700 hover:bg-blue-800 focus:ring-4 focus:ring-blue-300 font-medium rounded-lg text-sm px-5 py-2.5 mr-2 mb-2 dark:bg-blue-600 dark:hover:bg-blue-700 focus:outline-none dark:focus:ring-blue-800",
                    r#type: "submit",
                },
            }

        }
    ))
}

fn add_player(bracket: &UseSharedState<Bracket>, e: Event<FormData>) {
    let name = e.values.get("name").expect("name");
    let name = if name.is_empty() {
        let i = bracket.read().get_participants().len() + 1;
        format!("player {}", i)
    } else {
        name.to_string()
    };
    let b = match bracket.write().clone().add_participant(&name){ 
        Ok(b) => b,
        Err(_e) => panic!("oh no"),
    };
   
    *bracket.write() = b;
}

pub fn View(cx: Scope) -> Element {
    let format = match use_shared_state::<Bracket>(cx) {
        Some(bracket_ref) => bracket_ref.read().get_format(),
        None => Bracket::default().get_format(),
    };
    let view = match format {
        Format::SingleElimination => SingleEliminationBracketView(cx),
        Format::DoubleElimination => DoubleEliminationBracketView(cx),
    };
    
    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Bracket view"
        }
        view

    ))
}

#[derive(PartialEq, Clone)]
struct SomeProps {
    id: &'static MatchId,
}

fn display_match(cx: Scope, m: DisplayableMatch) -> Element {
    let m_id = use_shared_state::<Option<MatchId>>(cx).expect("match id");

    let start = match m.row_hint {
        Some(h) => format!("row-start-{}", h + 1),
        None => "".into(),
    };
    
    cx.render(rsx!(
        div {
            id: "{m.id}",
            onclick: move |_| {
                *m_id.write() = Some(m.id);
            },

            class: "col-span-1 flex flex-col my-auto box-border border-2 {start}",

            // TODO format seed display ### so it takes the same space for all
            div {
                class: "grow flex flex-row",
                div { format!("({})", m.seeds[0]) }
                div {
                    class: "box-border border grow",
                    m.player1()
                }
                div {
                    class: "max-width: 15px; box-border border",
                    m.score1()
                }
            }
            div {
                class: "grow flex flex-row",
                div { format!("({})", m.seeds[1]) }
                div {
                    class: "box-border border grow",
                    m.player2()
                }
                div {
                    class: "max-width: 15px; box-border border",
                    m.score2()
                }
            }
        }
    ))
}

fn SingleEliminationBracketView(cx: Scope) -> Element {
    // let mut bracket = use_state(cx, || bracket);
    let bracket = match use_shared_state::<Bracket>(cx) {
        Some(bracket_ref) => bracket_ref.read().clone(),
        None => Bracket::default(),
    };
    
    let matches = bracket.get_matches();
    let participants = bracket.get_participants();
    
    let mut rounds = winner_bracket(matches, &participants);
    reorder_rounds(&mut rounds);
    
    // NOTE: given a number of players, the number of the matches is know
    // Then I can deal with an array of fixed size for the matches. It's not
    // like switching from Vec to array would hurt me, now would it?
    
    // TODO finish this code before next job maybe
    let columns = rounds.len();
    
    return cx.render(rsx!( 
        div {
            // 128 = 2^7
            // 4096 = 2^12
            class: "grid grid-rows-1 grid-cols-{columns}",
            
            for (i, round) in rounds.iter().enumerate() {
                match i {
                    i if i == 0 => rsx!(
                        div {
                            class: "grid grid-cols-1 grow grid-flow-row",
                            round.iter().map(|m| display_match(cx, *m))
                        }
                    ),
                    _ => rsx!(
                        div {
                            class: "grid grid-cols-1",
                            round.iter().map(|m| display_match(cx, *m))
                        }
                    ),
                }
            }
        }
    ))
}

fn DoubleEliminationBracketView(cx: Scope) -> Element {
    // let n = bracket.get_participants().len();
    // TODO partition matches according to powers of two
    // let matches = bracket.get_matches();
    
    cx.render(rsx!( ""
        // p { n.to_string() }
        // for m in matches.iter() {
            // p { m.to_string() }
        // }
    ))
}

