#![allow(non_snake_case)]

use totsugeki::{
    bracket::Bracket, 
    matches::Id as MatchId, opponent::Opponent
};
use dioxus::prelude::*;

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
    
    let b = match b.tournament_organiser_reports_result(p1, result, p2){
        Ok(b) => b.0 ,
        Err(e) => {
            println!("{e}"); // TODO use a logging library
            return;
        }       
    };
    *bracket.write() = b;
}