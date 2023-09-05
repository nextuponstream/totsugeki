// FIXME works but no lsp => no auto complete and no refactor
#![no_main]

use libfuzzer_sys::fuzz_target;
extern crate libfuzzer_sys;

use chrono::prelude::*;
use totsugeki::{
    bracket::Bracket, format::Format, matches::ReportedResult, opponent::Opponent, player::Player,
    seeding::Method,
};
use totsugeki_fuzz::{BracketFormat, Events, MatchEvent};

fuzz_target!(|data: (Events, BracketFormat)| {
    let (events, format) = data;

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

    let format = match format {
        BracketFormat::SingleElimination => Format::SingleElimination,
        BracketFormat::DoubleElimination => Format::DoubleElimination,
    };

    let mut bracket = Bracket::new(
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
        bracket = bracket.join(player).expect("");
    }

    // println!("{format:?}");
    // println!("#total players: {total_players}");
    // println!("#events: {}", events.0.len());
    bracket = bracket.start().expect("bracket started").0;
    let mut dq_events = 0;

    // println!("#matches: {}", bracket.get_matches().len());

    for (i, e) in events.sequence.iter().enumerate() {
        // println!("{e:?}");
        // println!("{}", bracket.summary());
        if bracket.is_over() {
            break;
        }
        let matches = bracket.get_matches();
        // We processed enough events
        if matches.len() <= i {
            assert!(
                bracket.is_over(),
                "expected bracket to be over, {}",
                bracket.summary()
            );
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
                    (true, [Opponent::Player(id), _]) => id,
                    (false, [_, Opponent::Player(id)]) => id,
                    _ => {
                        panic!("cannot disqualify: {}", bracket.summary());
                    }
                };
                if dq_events < events.sequence.len() {
                    bracket = bracket.disqualify_participant(player).expect("bracket").0;
                } else {
                    assert!(
                        bracket.is_over(),
                        "expected bracket to be over {}",
                        bracket.summary()
                    );
                    return;
                }
            }
            MatchEvent::TOWin(is_player_1) => {
                let (p1, p2) = match m.get_players() {
                    [Opponent::Player(p1), Opponent::Player(p2)] => (p1, p2),
                    _ => panic!("oh no"),
                };
                let mut result = ReportedResult((2, 0));
                if !*is_player_1 {
                    result = result.reverse();
                }
                if !bracket.is_over() {
                    bracket = match bracket.tournament_organiser_reports_result(p1, result.0, p2) {
                        Ok((b, _, _)) => b,
                        Err(e) => panic!("TO can't report result: {e}"),
                    };
                }
            }
        }
    }

    assert!(
        bracket.is_over(),
        "expected bracket to be over: {}",
        bracket.summary()
    );
});
