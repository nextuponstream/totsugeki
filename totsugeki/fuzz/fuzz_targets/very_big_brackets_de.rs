#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate libfuzzer_sys;

use chrono::prelude::*;
use itertools::Itertools;
use num_bigint::BigInt;
use totsugeki::{
    bracket::Bracket, format::Format, matches::ReportedResult, opponent::Opponent, player::Player,
    seeding::Method,
};
use totsugeki_fuzz::{BracketFormat, ExtremeLotsOfEvents, MatchEvent};

// Fuzz for 2100 players, realistic in player size but EXTREMELY SLOW TO FUZZ
fuzz_target!(|data: (ExtremeLotsOfEvents, u128)| {
    let (events, permutation_index) = data;

    let total_events = events.0.len();

    let total_players = (total_events + 1) / 2; // 2 * n - 1 = t_e

    let format = Format::DoubleElimination;

    let mut min_permutations: BigInt = 5.into();
    let mut min_player_count = 3;
    let p_index_big_int = <u128 as Into<BigInt>>::into(permutation_index);

    for player_count in 3..total_players {
        if min_permutations < p_index_big_int {
            let next = player_count * 2 - 1;
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
        let event_count = 2 * player_count - 1;
        let mut bracket = Bracket::new(
            "",
            format,
            Method::Strict,
            Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap(),
            true,
        );

        let mut players = vec![Player::new(String::default())];
        for i in 1..=player_count {
            let player = Player::new(format!("p{i}"));
            players.push(player.clone());
            bracket = bracket.join(player).expect("");
        }
        let mut bracket = bracket.start().expect("bracket started").0;

        let mut permutations = (0..event_count).permutations(event_count);

        // there is always at least 2 permutations (3 players, single elim)
        // computing factorials is heavy.
        // when there is not enough player to use given permutation index, then
        // skip iteration `player_count`
        // otherwise, take the next iteration

        let p = permutations
            .nth(permutation_index.try_into().unwrap())
            .expect("permutation");

        for _ in 0..event_count {
            // early exit if there is not enough matches to fuzz
            let mut dq_events = 0;
            if bracket.is_over() {
                break;
            }

            for index_event in p.iter() {
                let e = events.0.get(*index_event).expect("event");
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
                        if dq_events < events.0.len() {
                            if !bracket.is_disqualified(player) {
                                dq_events = dq_events + 1;
                                bracket =
                                    bracket.disqualify_participant(player).expect("bracket").0;
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
                            bracket = match bracket
                                .tournament_organiser_reports_result(p1, result.0, p2)
                            {
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
    }
});
