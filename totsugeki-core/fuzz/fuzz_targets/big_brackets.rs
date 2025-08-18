#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate libfuzzer_sys;

use itertools::Itertools;
use num_bigint::BigInt;
use totsugeki_core::matches::result::Score;
use totsugeki_core::{matches::ReportedResult, opponent::Opponent};
use totsugeki_fuzz::{get, BracketFormat, LotsOfEvents, MatchEvent};

// NOTE: usize, 22! is the max
// iterations are long:
// ALARM: working on the last Unit for 1201 seconds
//        and the timeout value is 1200 (use -timeout=N to change)
fuzz_target!(|data: (LotsOfEvents, BracketFormat, u128)| {
    let (events, format, permutation_index) = data;

    let total_events = events.0.len();

    let total_players = match (format, total_events) {
        (BracketFormat::SingleElimination, t_e) => t_e + 1, // n - 1 = t_e
        (BracketFormat::DoubleElimination, t_e) => (t_e + 1) / 2, // 2 * n - 1 = t_e
    };

    let mut min_permutations: BigInt = match format {
        BracketFormat::SingleElimination => 2.into(),
        BracketFormat::DoubleElimination => 5.into(),
    };
    let mut min_player_count = 3;
    let p_index_big_int = <u128 as Into<BigInt>>::into(permutation_index);

    for player_count in 3..total_players {
        if min_permutations < p_index_big_int {
            let next = match format {
                BracketFormat::SingleElimination => player_count,
                BracketFormat::DoubleElimination => player_count * 2 - 1,
            };
            min_permutations = min_permutations * <usize as Into<BigInt>>::into(next);
            min_player_count = player_count;
        } else {
            break;
        }
    }

    // iterate over a growing player count
    for player_count in 3..total_players {
        if player_count < min_player_count {
            continue;
        }

        // required events in this loop
        let event_count = match format {
            BracketFormat::SingleElimination => player_count - 1,
            BracketFormat::DoubleElimination => 2 * player_count - 1,
        };

        let (mut seb, mut deb) = get(format, player_count);

        let mut permutations = (0..event_count).permutations(event_count);

        // there is always at least 2 permutations (3 players, single elim)
        // computing factorials is heavy.
        // when there is not enough player to use given permutation index, then
        // skip iteration `player_count`
        // otherwise, take the next iteration

        let p = permutations
            .nth(permutation_index.try_into().unwrap())
            .expect("permutation");

        println!("format                    : {format:?}");
        println!("player count for this loop: {player_count}");
        println!("permutation               : {permutation_index}");
        println!("-------------------------------");

        for _ in 0..event_count {
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
                let e = events.0.get(*index_event).expect("event");
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
                    BracketFormat::SingleElimination => {
                        seb.as_ref().unwrap().get_matches().to_owned()
                    }
                    BracketFormat::DoubleElimination => {
                        deb.as_ref().unwrap().get_matches().to_owned()
                    }
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
                        if dq_events < events.0.len() {
                            match format {
                                BracketFormat::SingleElimination => {
                                    if !seb.as_ref().unwrap().is_disqualified(player) {
                                        dq_events = dq_events + 1;
                                        seb = Some(
                                            seb.unwrap()
                                                .disqualify_participant_from_bracket(player)
                                                .0,
                                        );
                                    }
                                }
                                BracketFormat::DoubleElimination => {
                                    if !deb.as_ref().unwrap().is_disqualified(player) {
                                        dq_events = dq_events + 1;
                                        deb = Some(
                                            deb.unwrap()
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
                                        seb.unwrap()
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
                                        deb.unwrap()
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
    }
});
