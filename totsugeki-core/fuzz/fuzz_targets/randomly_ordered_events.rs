#![no_main]

extern crate libfuzzer_sys;
use libfuzzer_sys::fuzz_target;

use itertools::Itertools;
use totsugeki_core::matches::result::Score;
use totsugeki_core::{matches::ReportedResult, opponent::Opponent};
use totsugeki_fuzz::{get, BracketFormat, EventsPermutation, MatchEvent};

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

    println!("{format:?}");
    println!("#total players: {total_players}");
    println!("#events       : {}", total_events);
    println!("#permutation  : {}", events.permutation);
    println!("-------------------");
    let initial_bracket = get(format, total_players);

    let permutations = (0..total_events)
        .into_iter()
        .permutations(total_events)
        .collect_vec();
    let p = permutations.get(events.permutation).expect("permutation");
    let mut bracket = initial_bracket.clone();
    let (mut seb, mut deb) = (bracket.0, bracket.1);

    for _ in 0..total_events {
        // early exit if there is not enough matches to fuzz
        let mut dq_events = 0;
        match format {
            BracketFormat::SingleElimination => {
                if seb.as_ref().unwrap().is_over() {
                    break;
                }
            }
            BracketFormat::DoubleElimination => {
                if deb.as_ref().unwrap().is_over() {
                    break;
                }
            }
        }

        for index_event in p.iter() {
            let e = events.sequence.get(*index_event).expect("event");
            match format {
                BracketFormat::SingleElimination => {
                    if seb.as_ref().unwrap().is_over() {
                        break;
                    }
                }
                BracketFormat::DoubleElimination => {
                    if deb.as_ref().unwrap().is_over() {
                        break;
                    }
                }
            }
            let matches = match format {
                BracketFormat::SingleElimination => seb.as_ref().unwrap().get_matches(),
                BracketFormat::DoubleElimination => deb.as_ref().unwrap().get_matches(),
            };
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
                        (true, [Opponent(Some(id)), _]) => id,
                        (false, [_, Opponent(Some(id))]) => id,
                        _ => {
                            continue;
                        }
                    };
                    if dq_events < events.sequence.len() {
                        match format {
                            BracketFormat::SingleElimination => {
                                if !seb.as_ref().unwrap().is_disqualified(player) {
                                    dq_events = dq_events + 1;
                                    seb = Some(
                                        seb.clone()
                                            .unwrap()
                                            .disqualify_participant_from_bracket(player)
                                            .0,
                                    );
                                }
                            }
                            BracketFormat::DoubleElimination => {
                                if !deb.as_ref().unwrap().is_disqualified(player) {
                                    dq_events = dq_events + 1;
                                    deb = Some(
                                        deb.clone()
                                            .unwrap()
                                            .disqualify_participant_from_bracket(player)
                                            .unwrap()
                                            .0,
                                    );
                                }
                            }
                        }
                    } else {
                        match format {
                            BracketFormat::SingleElimination => {
                                assert!(
                                    seb.as_ref().unwrap().is_over(),
                                    "expected bracket to be over {}",
                                    seb.as_ref().unwrap().summary(),
                                );
                            }
                            BracketFormat::DoubleElimination => {
                                assert!(
                                    deb.as_ref().unwrap().is_over(),
                                    "expected bracket to be over {}",
                                    deb.as_ref().unwrap().summary(),
                                );
                            }
                        }
                        break;
                    }
                }
                MatchEvent::TOWin(is_player_1) => {
                    let (p1, p2) = match m.get_players() {
                        [Opponent(Some(p1)), Opponent(Some(p2))] => (p1, p2),
                        _ => continue,
                    };
                    let mut result = ReportedResult(Some(Score(2, 0)));
                    if !is_player_1 {
                        result = result.reverse();
                    }
                    match format {
                        BracketFormat::SingleElimination => {
                            if !seb.as_ref().unwrap().is_over() {
                                seb = Some(
                                    seb.clone()
                                        .unwrap()
                                        .tournament_organiser_reports_result(
                                            p1,
                                            result.0.unwrap(),
                                            p2,
                                        )
                                        .unwrap()
                                        .0,
                                )
                            }
                        }
                        BracketFormat::DoubleElimination => {
                            if !deb.as_ref().unwrap().is_over() {
                                deb = Some(
                                    deb.clone()
                                        .unwrap()
                                        .tournament_organiser_reports_result_dangerous(
                                            p1,
                                            result.0.unwrap(),
                                            p2,
                                        )
                                        .unwrap()
                                        .0,
                                )
                            }
                        }
                    }
                }
            }
        }
    }

    match format {
        BracketFormat::SingleElimination => {
            assert!(
                seb.as_ref().unwrap().is_over(),
                "expected bracket to be over {}",
                seb.as_ref().unwrap().summary(),
            );
        }
        BracketFormat::DoubleElimination => {
            assert!(
                deb.as_ref().unwrap().is_over(),
                "expected bracket to be over {}",
                deb.as_ref().unwrap().summary(),
            );
        }
    }
});
