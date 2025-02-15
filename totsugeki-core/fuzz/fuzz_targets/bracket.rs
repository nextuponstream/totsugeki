#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate libfuzzer_sys;

use totsugeki_core::bracket::seeding::Seeding;
use totsugeki_core::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki_core::matches::result::{MatchFormat, Score};
use totsugeki_core::single_elimination_bracket::SingleEliminationBracket;
use totsugeki_core::validation::AutomaticMatchValidationMode;
use totsugeki_core::{matches::ReportedResult, opponent::Opponent, player::Player, ID};
use totsugeki_fuzz::{get, BracketFormat, Events, MatchEvent};

fuzz_target!(|data: (Events, BracketFormat)| {
    let (events, format) = data;

    let debug = std::env::var("PRINTLN").is_ok();

    // early exit if there is not enough matches to fuzz
    match (format, events.sequence.len()) {
        (BracketFormat::SingleElimination, n) if n < 3 => {
            return;
        }
        (BracketFormat::DoubleElimination, n) if n < 5 => {
            return;
        }
        (_, _) => {}
    };

    let total_players = match (format, events.sequence.len()) {
        (BracketFormat::SingleElimination, i) if i <= 5 => events.sequence.len(),
        (BracketFormat::SingleElimination, _) => events.sequence.len() / 2,
        (BracketFormat::DoubleElimination, i) if i == 5 => 3, // 2 * n - 1 = total_matches
        (BracketFormat::DoubleElimination, _) => events.sequence.len() / 2, // 2 * n - 1 = total_matches
    };

    let (mut seb, mut deb) = get(format, total_players);

    if debug {
        println!("{format:?}");
        println!("#total players: {total_players}");
        println!("#events: {}", events.sequence.len());
        println!("{:?}", events.sequence);
    }
    let mut dq_events = 0;

    for (i, e) in events.sequence.iter().enumerate() {
        // println!("{e:?}");
        // println!("{}", bracket.summary());
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
            BracketFormat::SingleElimination => seb.as_ref().unwrap().get_matches().to_owned(),
            BracketFormat::DoubleElimination => deb.as_ref().unwrap().get_matches().to_owned(),
        };
        // if debug {
        //     for m in &matches {
        //         println!("{}", m.summary());
        //     }
        // }
        // We processed enough events
        if matches.len() <= i {
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
            return;
        }
        let m = matches.get(i).expect("match");

        // Some matches are already over because of a disqualification event.
        // Then there is no need to trigger an event on those.
        // println!("{:?} {}", m.get_seeds(), m.get_id());
        if m.is_over() {
            // println!("SKIPPED");
            continue;
        }
        match e {
            MatchEvent::Disqualification(is_player_1) => {
                dq_events = dq_events + 1;
                let player = match (is_player_1, m.get_players()) {
                    (true, [Opponent(Some(id)), _]) => id,
                    (false, [_, Opponent(Some(id))]) => id,
                    _ => {
                        unreachable!()
                    }
                };
                if events.sequence.len() < 40 {
                    let pos = match format {
                        BracketFormat::SingleElimination => {
                            seb.as_ref()
                                .unwrap()
                                .get_seeding()
                                .get()
                                .iter()
                                .position(|s| s == player)
                                .unwrap()
                                + 1
                        }
                        BracketFormat::DoubleElimination => {
                            deb.as_ref()
                                .unwrap()
                                .get_seeding()
                                .get()
                                .iter()
                                .position(|s| s == player)
                                .unwrap()
                                + 1
                        }
                    };
                    if debug {
                        println!("disqualify player {} with ID {}", pos, player);
                    }
                }
                // <= because 4 dq in 5 person tournaments (single elimination) with 4 events...
                if dq_events <= events.sequence.len() {
                    match format {
                        BracketFormat::SingleElimination => {
                            seb = Some(seb.unwrap().disqualify_participant_from_bracket(player).0);
                        }
                        BracketFormat::DoubleElimination => {
                            deb = Some(
                                deb.unwrap()
                                    .disqualify_participant_from_bracket(player)
                                    .unwrap()
                                    .0,
                            );
                        }
                    }
                } else {
                    // println!("{}", dq_events);
                    // println!("{}", events.sequence.len());
                    let events = if events.sequence.len() < 40 {
                        format!("{events:?}")
                    } else {
                        format!("... too many events to list ({})", events.sequence.len())
                    };
                    match format {
                        BracketFormat::SingleElimination => {
                            assert!(
                                seb.as_ref().unwrap().is_over(),
                                "expected bracket to be over {}\n{events}",
                                seb.as_ref().unwrap().summary(),
                            );
                        }
                        BracketFormat::DoubleElimination => {
                            assert!(
                                deb.as_ref().unwrap().is_over(),
                                "expected bracket to be over {}\n{events}",
                                deb.as_ref().unwrap().summary(),
                            );
                        }
                    }
                    return;
                }
            }
            MatchEvent::TOWin(is_player_1) => {
                let (p1, p2) = match m.get_players() {
                    [Opponent(Some(p1)), Opponent(Some(p2))] => (p1, p2),
                    _ => panic!("oh no"),
                };
                let mut result = ReportedResult(Some(Score(2, 0)));
                if !*is_player_1 {
                    result = result.reverse();
                }
                match format {
                    BracketFormat::SingleElimination => {
                        if !seb.as_ref().unwrap().is_over() {
                            seb = Some(
                                seb.unwrap()
                                    .tournament_organiser_reports_result(p1, result.0.unwrap(), p2)
                                    .unwrap()
                                    .0,
                            );
                        }
                    }
                    BracketFormat::DoubleElimination => {
                        if !deb.as_ref().unwrap().is_over() {
                            deb = Some(
                                deb.unwrap()
                                    .tournament_organiser_reports_result_dangerous(
                                        p1,
                                        result.0.unwrap(),
                                        p2,
                                    )
                                    .unwrap()
                                    .0,
                            );
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
