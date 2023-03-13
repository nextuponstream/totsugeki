#![allow(non_snake_case)]

use std::cmp::min;
use chrono::{TimeZone, Utc};
use dioxus::prelude::*;
use totsugeki::{
    bracket::Bracket, 
    format::Format, 
    matches::{Match, Id as MatchId}, 
    player::{Participants, Id as PlayerId}
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

        form {
            onsubmit: move |event| { update_bracket(bracket, event ) },

            div {
                class: "pb-2",
                label { "Name" }
                input {
                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
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
                    class: "bg-gray-50 border border-gray-300 text-gray-900 text-sm rounded-lg focus:ring-blue-500 focus:border-blue-500 block p-2.5 dark:bg-gray-700 dark:border-gray-600 dark:placeholder-gray-400 dark:text-white dark:focus:ring-blue-500 dark:focus:border-blue-500",
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
    
    let bracket = use_shared_state::<Bracket>(cx).expect("bracket");
    
    let b = bracket.read();
    
    let view = match b.get_format() {
        Format::SingleElimination => SingleEliminationBracketView(cx, b.clone()),
        Format::DoubleElimination => DoubleEliminationBracketView(cx, b.clone()),
    };

    cx.render(rsx!(
        h2 {
            class: "text-lg",
            "Bracket view"
        }
        view.unwrap()

    ))
}

fn SingleEliminationBracketView(cx: Scope, bracket: Bracket) -> Element {
        // Any trace events in this closure or code called by it will occur within
        // the span.
    // TODO partition matches according to powers of two
    let matches = bracket.get_matches();
    let participants = bracket.get_participants();
    // let players: Vec<(PlayerId, &str)> = participants.get_players_list().iter().map(|p| (p.get_id(), p.get_name().as_str())).collect();
    
    // participants.get(id).unwrap().get_name()
    
    let rounds = partition_winner_bracket(matches, &participants);
    
    // NOTE: given a number of players, the number of the matches is know
    // Then I can deal with an array of fixed size for the matches. It's not
    // like switching from Vec to array would hurt me, now would it?
    
    // TODO finish this code before next job maybe
    
    // TODO display player names
    // option 1, provide name in method
    // tried that and got HURT
    // option 2, rewrite opponent to include player name (byte vector?)
        // for round in rounds {
        //     for m in round {
        //         let (p1, p2) = m.get_players();
        //         p { p1.to_string() }
        //         p { p2.to_string() }
        //     }
        // }
    
    // TODO check display when 12 rounds are in play
    let columns = min(rounds.len(), 12);
    
    
    return cx.render(rsx!( 

       
        div { "rounds: {columns}" }
        div {
            // TODO columns is "correct" but alignement is not
            // TODO parameterize rows-X
            class: "grid grid-rows-1 grid-cols-{columns} box-border border-4",
            // div {"1"} div {"2"}
            rounds.iter().map(move |round| rsx!( 
                div {
                    class: "grid grid-cols-1 grid-rows-{round.len()}",
            
                    round.iter().map(|m| rsx!(
                        div {
                            class: "col-span-1",
                            div {
                                class: "container columns-2 box-border border-2",
                                div {
                                    m.player1()
                                }
                                div {
                                    class: "max-width: 15px; box-border border-1",
                                    "0"
                                }
                            }
                            div {
                                class: "container columns-2 box-border border-2",
                                div {
                                    m.player2()
                                }
                                div {
                                    class: "max-width: 15px; box-border border-1",
                                    "0"
                                }
                            }
                        }
                    ))
                }
            ))
        }
    ))
}

fn DoubleEliminationBracketView(cx: Scope, bracket: Bracket) -> Element {
    let n = bracket.get_participants().len();
    // TODO partition matches according to powers of two
    let matches = bracket.get_matches();
    
    cx.render(rsx!(
        p { n.to_string() }
        for m in matches.iter() {
            p { m.to_string() }
        }
    ))
}

#[derive(Clone, Copy, Debug)]
struct DisplayableMatch {
    id: MatchId,
    players: [[u8; 256]; 2]
}

impl DisplayableMatch {
    fn player(&self, is_player1: bool) -> &str {
        let id = if is_player1 {0} else {1};
        std::str::from_utf8(&self.players[id]).unwrap()    
    }
    
    fn player1(&self) -> &str {
        self.player(true)
    }
    
    fn player2(&self) -> &str {        
        self.player(false)
    }
}

// TODO remove, not actually using this
impl std::fmt::Display for DisplayableMatch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} VS {}", self.player1(), self.player2())
    }
}

fn convert(m: &Match, participants: &Participants) -> DisplayableMatch {
    let list = participants.get_players_list();
    let players: Vec<(PlayerId, String)> = list.iter().map(|p| (p.get_id(), p.get_name())).collect();
    let player_name_size = 256;
    let mut player1_name = 
        m.get_players()[0].get_name(&players).into_bytes().into_iter().take(256).collect::<Vec<u8>>();
    player1_name.resize(player_name_size, 0); // '\0' null byte
    let player1 = player1_name.try_into().unwrap();
    let mut player2_name = 
        m.get_players()[1].get_name(&players).into_bytes().into_iter().take(256).collect::<Vec<u8>>();
    player2_name.resize(player_name_size, 0); // '\0' null byte
    let player2 = player2_name.try_into().unwrap();
    DisplayableMatch { id: m.get_id(), players: [player1, player2] }
}

fn partition_winner_bracket(matches: Vec<Match>, participants: &Participants) -> Vec<Vec<DisplayableMatch>> {
    // TODO split matches in descending powers of two groups
    let n = participants.len();
    let Some(mut npo2) =  n.checked_next_power_of_two() else {
        panic!("MATH");
    };
    let matches: Vec<DisplayableMatch> = matches.iter().map(|m| convert(m, participants)).collect();
    let byes = npo2 - n;
    let mut remaining_matches = matches;
    let mut partition = vec![];
    let mut is_first_round = true;
    while !remaining_matches.is_empty() {
        if is_first_round {
            is_first_round = false;
            // 5 players, 2^npo2 >= 5 -> npo2 = 3
            // byes = npo2 - 5 = 3
            // 2³ = 8
            // 3 players dont play
            // 2 players have to play
            // 1 match
            // next round
            // 4 players => 2 matches
            // 2 players => 1 match
            
            // 4 players, 2^npo2 >= 4 -> npo2 = 2
            // byes = npo2 - 4 = 0
            // 2² = 4
            // 2 players don't play
            // no
            // 4 players, 2^byes == #participants -> byes = 0
            // 4 players, 2 matches
            // 2 players, 1 match
            
            // 3 players, 2^byes > #participants (3) -> npo2 = 2
            // byes = 4 - 3 = 1
            // 1 players does not play
            // 2 players play
            // 1 match
            // 2 players play
            // 1 match
            
            let remaining_players = participants.len() - byes;
            let split = remaining_players / 2;
            // TODO use drain
            let tmp = remaining_matches.clone();
            let (first_round_matches, matches) = tmp.split_at(split);
            remaining_matches = matches.to_vec();
            partition.push(first_round_matches.to_vec());
            continue;
        } else {
            npo2 /= 2;
            let split = npo2 / 2;
            let (round, matches) = if remaining_matches.len() == 1 {
                // NOTE: I really don't like the unwrap but assigning
                // `remaining_matches` to an empty vec produces a warning
                // TODO remove unwrap
                let tmp = remaining_matches.drain(0..1).next().unwrap();
                (vec![tmp], vec![])
            } else {
                let (a, b) = remaining_matches.split_at(split);
                (a.to_vec(), b.to_vec())
            };
            partition.push(round.to_vec());
            remaining_matches = matches.to_vec();
            continue;
        }
    }
    
    partition
}

#[cfg(test)]
mod tests {
    use totsugeki::{matches::Match, player::{Participants, Player}};

    use super::partition_winner_bracket;
    
    fn get_matches_and_participant(n: usize) -> (Vec<Match>, Participants) {
        let mut matches = vec![];
        let mut players = vec![];
        for _ in 0..n {
            matches.push(Match::default());
        }
        for i in 1..=n {
            players.push(Player::new(format!("p{i}")));
        }
        let participants = Participants::try_from(players).expect("participants");
        (matches, participants)
    }
    
    #[test]
    fn split_winner_bracket_3_participants() {
        let (matches, participants) = get_matches_and_participant(3);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 1, "first round");
        assert_eq!(partition[1].len(), 1, "second round");
    }

    #[test]
    fn split_winner_bracket_4_participants() {
        let (matches, participants) = get_matches_and_participant(4);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 2, "first round, 1-4 + 2-3");
        assert_eq!(partition[1].len(), 1, "second round, 1-2");
    }

    #[test]
    fn split_winner_bracket_5_participants() {
        let (matches, participants) = get_matches_and_participant(5);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 1, "first round, 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_6_participants() {
        let (matches, participants) = get_matches_and_participant(6);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 2, "first round, 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_7_participants() {
        let (matches, participants) = get_matches_and_participant(7);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 3, "first round, 2-7 + 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_8_participants() {
        let (matches, participants) = get_matches_and_participant(8);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[1].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[2].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_9_participants() {
        let (matches, participants) = get_matches_and_participant(9);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 1, "first round, 8-9");
        assert_eq!(partition[1].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[2].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[3].len(), 1, "third round, 1-2");
    }

    #[test]
    fn split_winner_bracket_10_participants() {
        let (matches, participants) = get_matches_and_participant(10);
        let partition = partition_winner_bracket(matches, &participants);
        
        assert_eq!(partition[0].len(), 2, "first round, 7-10 + 8-9");
        assert_eq!(partition[1].len(), 4, "first round, 1-8 + 2-7 + 3-6 + 4-5");
        assert_eq!(partition[2].len(), 2, "second round, 1-4 + 2-3");
        assert_eq!(partition[3].len(), 1, "third round, 1-2");
    }
}
