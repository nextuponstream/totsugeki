#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate libfuzzer_sys;

use chrono::prelude::*;
use totsugeki::{
    bracket::Bracket, format::Format, matches::ReportedResult, opponent::Opponent, player::Player,
    seeding::Method,
};
use totsugeki_fuzz::{BracketFormat, MatchEvent, StillRealisticEvents};

// Note: for 25k players, this fuzzing target hits catastrophic scenarios
// quite easily. Even 1k player for double elimination, this hits catastrophic
// scenarios.
fuzz_target!(|data: (StillRealisticEvents, BracketFormat)| {
    let (events, format) = data;
    // let (events, _format) = data;
    let total_events = events.sequence.len();

    // if you want to temporarily force another type of bracket format, update
    // format here (and not below)
    // let format = BracketFormat::DoubleElimination;
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

    for i in 1..=total_players {
        // if i % 100 == 0 {
        //     println!("{i} player joined...")
        // }
        let player = Player::new(format!("p{i}"));
        // unchecked does not slow down
        initial_bracket = initial_bracket
            .unchecked_join_skip_matches_generation(player)
            .expect("bracket with more participants but no matches");
    }

    initial_bracket = initial_bracket.generate_matches().expect("matches");

    println!("{format:?}");
    println!("#total players: {total_players}");
    println!("#events       : {}", total_events);
    // println!("#permutation  : {}", events.permutation);
    println!("-------------------");
    let initial_bracket = initial_bracket.start().expect("bracket started").0;

    // cannot compute all permutations for big brackets because computationnaly
    // expensive. Then take random sequence provided
    let p = events.permutation;

    let mut bracket = initial_bracket;

    for _e in 0..total_events {
        // if e > 0 && e % 50 == 0 {
        //     println!("{e} events...");
        // }
        // early exit if there is not enough matches to fuzz
        let mut dq_events = 0;
        if bracket.is_over() {
            break;
        }

        // FIXME it takes a full second to process 1 event for 8k people
        for index_event in p.iter() {
            // println!("{}/{}", index_event, p.len());
            if *index_event > 0 && *index_event % 20 == 0 {
                // println!("ie {index_event}...");
                let (done, total) = bracket.matches_progress();
                println!("{done}/{total} matches done");
                // let ms = bracket.get_matches();
                // println!("{:?}", ms[ms.len() - 1]);
                // println!("{:?}", ms[ms.len() - 2]);
            }
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
            // println!("processing event {e:?} for match {m:?}");
            match e {
                MatchEvent::Disqualification(is_player_1) => {
                    // println!("{e:?}");
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
                            // if dq_events % 500 == 0 {
                            //     println!("processed {dq_events} disqualifications...");
                            // }
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
