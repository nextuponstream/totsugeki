#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate libfuzzer_sys;

use chrono::prelude::*;
use itertools::Itertools;
use totsugeki::{
    bracket::Bracket, format::Format, matches::ReportedResult, opponent::Opponent, player::Player,
    seeding::Method,
};
use totsugeki_fuzz::{BracketFormat, EventsPermutation, MatchEvent};

// NOTE: fuzzer is stuck between tournaments of 3-10 players
fuzz_target!(|data: (EventsPermutation, BracketFormat)| {
    let (events, format) = data;
    let total_events = events.sequence.len();

    match (format, total_events) {
        (BracketFormat::SingleElimination, t_e) if t_e < 3 => {
            return;
        }
        (BracketFormat::DoubleElimination, t_e) if t_e < 5 => {
            return;
        }
        (_, t_e) if t_e % 2 == 0 => {
            return;
        }
        (_, _) => {}
    };
    let total_players = match (format, total_events) {
        (BracketFormat::SingleElimination, t_e) => t_e + 1, // n - 1 = t_e
        (BracketFormat::DoubleElimination, t_e) => (t_e + 1) / 2, // 2 * n - 1 = t_e
    };

    let format = match format {
        BracketFormat::SingleElimination => Format::SingleElimination,
        BracketFormat::DoubleElimination => Format::DoubleElimination,
    };

    let mut initial_bracket = Bracket::new(
        "",
        format,
        Method::Strict,
        Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
        true,
    );

    let mut players = vec![Player::new(String::default())];
    for i in 1..=total_players {
        let player = Player::new(format!("p{i}"));
        players.push(player.clone());
        initial_bracket = initial_bracket.join(player).expect("");
    }

    println!("{format:?}");
    println!("#total players: {total_players}");
    println!("#events       : {}", total_events);
    println!("#permutation  : {}", events.permutation);
    println!("-------------------");
    let initial_bracket = initial_bracket.start().expect("bracket started").0;

    let permutations = (0..total_events)
        .into_iter()
        .permutations(total_events)
        .collect_vec();
    let p = permutations.get(events.permutation).expect("permutation");
    let mut bracket = initial_bracket.clone();

    for _ in 0..total_events {
        // early exit if there is not enough matches to fuzz
        let mut dq_events = 0;
        if bracket.is_over() {
            break;
        }

        for index_event in p.iter() {
            let e = events.sequence.get(*index_event).expect("event");
            if bracket.is_over() {
                break;
            }
            let matches = bracket.get_matches();
            // there may be too many events, then skip
            if *index_event >= matches.len() {
                continue;
            }
            let m = matches.get(*index_event).expect("match");

            // Some matches are already over because of a disqualification event.
            // Then there is no need to trigger an event on those.
            if m.is_over() {
                continue;
            }
            match e {
                MatchEvent::Disqualification(is_player_1) => {
                    let player = match (is_player_1, m.get_players()) {
                        (true, [Opponent::Player(id), _]) => id,
                        (false, [_, Opponent::Player(id)]) => id,
                        _ => {
                            continue;
                        }
                    };
                    if dq_events < events.sequence.len() {
                        if !bracket.is_disqualified(player) {
                            dq_events = dq_events + 1;
                            bracket = bracket.disqualify_participant(player).expect("bracket").0;
                        }
                    } else {
                        assert!(
                            bracket.is_over(),
                            "expected bracket to be over {}",
                            bracket.summary()
                        );
                        break;
                    }
                }
                MatchEvent::TOWin(is_player_1) => {
                    let (p1, p2) = match m.get_players() {
                        [Opponent::Player(p1), Opponent::Player(p2)] => (p1, p2),
                        _ => continue,
                    };
                    let mut result = ReportedResult((2, 0));
                    if !is_player_1 {
                        result = result.reverse();
                    }
                    if !bracket.is_over() {
                        bracket =
                            match bracket.tournament_organiser_reports_result(p1, result.0, p2) {
                                Ok((b, _, _)) => b,
                                Err(e) => panic!("TO can't report result: {e}"),
                            };
                    }
                }
            }
        }
    }

    assert!(
        bracket.is_over(),
        "all passes done but bracket is not over: {}",
        bracket.summary()
    );
});
