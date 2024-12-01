use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::matches::result::{MatchFormat, Score};
use totsugeki::matches::Match;
use totsugeki::opponent::Opponent;
use totsugeki::player::{Participants, Player, PlayerID};
use totsugeki::validation::AutomaticMatchValidationMode;

// Note: we don't test panics. If a panic occurs, it's a bug that needs
// programmer attention. If UI sends bad data to backend and violate assertions,
// then it's likely the UI is out of date and some synchronisation process is
// failing somewhere.
//
// Moving panics to errors will lead to enum error bloat and after doing a
// global enum with 17+ variants, I don't feel like dealing with too many
// unknowns. If violated assertions causes too many logged errors in production
// or error logs are too polluted with violated assertions (and dealing with it
// is not successful), then maybe treating violated assertions as an Error
// variant may be the way. But let's see if that's the case first before
//
// Example of panic testing?
// #[test]
// #[should_panic]
// fn disqualifying_unknown_player_panics() {
//     let mut participants = Participants::default();
//     for i in 1..=3 {
//         let player = Player::new(format!("p{i}"));
//         participants = participants.add_participant(player).expect("seeding");
//     }
//     let bracket = DoubleEliminationBracket::create(
//         Seeding::new(participants.get_seeding()).unwrap(),
//         AutomaticMatchValidationMode::Strict,
//     );
//
//     let unknown_player = ID::new_v4();
//     bracket
//         .disqualify_participant_from_bracket(unknown_player)
//         .unwrap();
// }

fn assert_player_drops_to_losers(bracket: &DoubleEliminationBracket, n: usize, p: &[Player]) {
    let (winners, losers, _, _) = bracket.partition_matches().expect("enough players");
    assert!(
        !winners.iter().any(|m| m.contains(p[n].get_id())
            && m.get_winner() == Opponent(None)
            && m.get_automatic_loser() == Opponent(None)),
        "expected player {n} having no matches in winners"
    );
    assert!(
        losers.iter().any(|m| m.contains(p[n].get_id())),
        "expected player {n} in losers",
    );
}

#[test]
fn disqualifying_player_that_could_not_make_it() {
    let mut p = vec![Player::new("don't use".into())];
    let mut participants = Participants::default();
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        participants = participants.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(participants.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(loser)) if loser == p[1].get_id())
        ),
        "expected player 1 not to be declared looser in any match"
    );
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[1].get_id())
        .expect("bracket with player 1 disqualified");
    assert!(
        bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(loser)) if loser == p[1].get_id())
        ),
        "expected match where player 1 is declared looser"
    );
    assert!(
        bracket
            .get_matches()
            .iter()
            .any(|m| m.contains(p[2].get_id()) && m.contains(p[3].get_id())),
        "expected player 2 and 3 playing"
    );
}

#[test]
fn disqualifying_player_sets_looser_of_their_current_match() {
    let mut p = vec![Player::new("don't use".into())];
    let mut participants = Participants::default();
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        participants = participants.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(participants.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Strict,
        MatchFormat::ft3(),
        None,
    );

    let (bracket, match_id_p2, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), Score(2, 0), p[3].get_id())
        .expect("reported result by player 2");
    let (bracket, _) = bracket.validate_match_result(match_id_p2);

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(loser)) if loser == p[2].get_id())
        ),
        "expected player 2 not to be declared looser in any match"
    );
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("p2 is disqualified");

    let p1_wins_against_p2_since_p2_is_disqualified =
        bracket
            .get_matches()
            .iter()
            .any(|m| match (m.get_automatic_loser(), m.get_winner()) {
                (Opponent(Some(loser)), Opponent(Some(winner))) if loser == p[2].get_id() => {
                    winner == p[1].get_id()
                }
                _ => false,
            });
    let fateful_match = bracket
        .clone()
        .get_matches()
        .into_iter()
        .find(|m| m.contains(p[1].get_id()) && m.contains(p[2].get_id()))
        .expect("m");
    assert!(
        p1_wins_against_p2_since_p2_is_disqualified,
        "expected player 1 winning match where player 2 is disqualified, got {}.\n{:?}",
        fateful_match.summary(),
        fateful_match
    );
    assert!(
        !bracket.is_over(),
        "as opposed to single elimination, bracket is not over"
    );
}

#[test]
fn disqualifying_player_sets_their_opponent_as_the_winner_and_they_move_to_their_next_match() {
    let mut p = vec![Player::new("don't use".into())];
    let mut participants = Participants::default();
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        participants = participants.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(participants.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Strict,
        MatchFormat::ft3(),
        None,
    );

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(loser)) if loser == p[2].get_id())
        ),
        "expected player 2 not to be declared looser in any match"
    );
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("bracket with player 2 disqualified");
    assert!(
        bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(player)) if player == p[2].get_id())
        ),
        "expected match where player 2 is declared looser"
    );
    assert!(
        bracket
            .get_matches()
            .iter()
            .any(|m| m.contains(p[1].get_id()) && m.contains(p[3].get_id())),
        "expected player 1 and 3 playing in winner finals"
    );
}

fn initial_step(
    n: usize,
    auto: AutomaticMatchValidationMode,
) -> (DoubleEliminationBracket, Vec<Player>) {
    let mut p = vec![Player::new("don't use".into())];
    let mut participants = Participants::default();
    for i in 1..=n {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        participants = participants.add_participant(player).expect("seeding");
    }
    (
        DoubleEliminationBracket::create(
            Seeding::new(participants.get_player_list()).unwrap(),
            auto,
            MatchFormat::ft3(),
            None,
        ),
        p,
    )
}

fn assert_outcome(bracket: &DoubleEliminationBracket, x: &Player, y: &Player) {
    assert!(
        bracket.get_matches().iter().any(|m| matches!((
                m.contains(x.get_id()),
                m.contains(y.get_id()),
                m.get_winner()
            ), (true, true, Opponent(Some(winner))) if winner == x.get_id())),
        "No match where {} wins against {}",
        x.get_name(),
        y.get_name()
    );
}
fn assert_outcome_in_matches(matches: &[Match], x: &Player, y: &Player) {
    assert!(
        matches.iter().any(|m| matches!((
                m.contains(x.get_id()),
                m.contains(y.get_id()),
                m.get_winner()
            ), (true, true, Opponent(Some(winner))) if winner == x.get_id())),
        "No match where {} wins against {}",
        x.get_name(),
        y.get_name()
    );
}

#[test]
fn disqualifying_everyone_is_impossible_because_the_last_player_remaining_wins_grand_finals_automatically(
) {
    let (bracket, p) = initial_step(8, AutomaticMatchValidationMode::Flexible);

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("p2 DQ'ed");
    assert_player_drops_to_losers(&bracket, 2, &p);
    assert_outcome(&bracket, &p[7], &p[2]);

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("p3 DQ'ed");
    assert_player_drops_to_losers(&bracket, 3, &p);
    assert_outcome(&bracket, &p[6], &p[3]);

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[4].get_id())
        .expect("p4 DQ'ed");
    assert_outcome(&bracket, &p[5], &p[4]);
    assert_player_drops_to_losers(&bracket, 4, &p);
    let (_, l_bracket, _, _) = bracket.partition_matches().expect("enough players");
    assert_eq!(
        l_bracket
            .iter()
            .filter(|m| m.contains(p[4].get_id()))
            .count(),
        1
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[5].get_id())
        .expect("p5 DQ'ed");
    // player 5 opponent in winners is unknown, yet he can drop to losers
    // already, even if 1vs8 has not been played out
    assert_player_drops_to_losers(&bracket, 5, &p);

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[6].get_id())
        .expect("p6 DQ'ed");
    assert_outcome(&bracket, &p[7], &p[6]);

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[7].get_id())
        .expect("p7 DQ'ed");
    assert_player_drops_to_losers(&bracket, 7, &p);
    assert!(&bracket
        .get_matches()
        .iter()
        .any(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3]));
    let (_w_bracket, l_bracket, _, _) = bracket.partition_matches().expect("enough players");
    let m = &l_bracket
        .iter()
        .find(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3])
        .expect("m");
    let Opponent(Some(loser)) = m.get_automatic_loser() else {
        panic!("expected loser but found none {m:?}");
    };
    assert_eq!(loser, p[7].get_id());

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[8].get_id())
        .expect("p8 DQ'ed");
    assert_outcome(&bracket, &p[1], &p[8]);
    assert_player_drops_to_losers(&bracket, 8, &p);
    assert_outcome(&bracket, &p[8], &p[5]);
    assert_outcome(&bracket, &p[1], &p[5]);
    assert_player_drops_to_losers(&bracket, 5, &p);
    assert_outcome(&bracket, &p[1], &p[7]);
    assert_player_drops_to_losers(&bracket, 7, &p);
    // player 7 is in GF
    (2..=8).for_each(|i| {
        assert!(
            bracket.is_disqualified(p[i].get_id()),
            "player {i} disqualified"
        );
    });
    assert!(
        !bracket.is_disqualified(p[1].get_id()),
        "player 1 not disqualified"
    );
    let (winner_bracket, loser_bracket, gf, _gf_reset) =
        bracket.partition_matches().expect("enough players");
    for m in &winner_bracket {
        assert_ne!(
            m.get_automatic_loser(),
            Opponent(None),
            "expected winner bracket match to have automatic loser but got none: {m:?}"
        );
    }
    for m in &loser_bracket {
        assert_ne!(
            m.get_automatic_loser(),
            Opponent(None),
            "expected loser bracket match to have automatic loser but got none: {m:?}"
        );
    }

    assert_outcome_in_matches(&winner_bracket, &p[1], &p[8]);
    assert_outcome_in_matches(
        &[*loser_bracket.last().expect("loser bracket finals")],
        &p[8],
        &p[7],
    );
    assert_outcome_in_matches(&[gf], &p[1], &p[8]);

    // https://stackoverflow.com/a/68919527
    assert!(matches!(
        bracket.disqualify_participant_from_bracket(p[1].get_id()),
        Err(totsugeki::double_elimination_bracket::disqualification::disqualify_from_bracket::Error::WonTournament)
    ));
}

#[test]
fn disqualifying_most_in_double_elimination_tournament_and_lowest_expected_seed_in_winners_final() {
    let mut players = vec![Player::new("don't use".into())];
    let mut p = vec![PlayerID::create()];
    let mut bad_seeding = Participants::default();
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        players.push(player.clone());
        p.push(player.get_id());
        bad_seeding = bad_seeding.add_participant(player).expect("seeding");
    }
    let seeding = bad_seeding
        .get_players_list()
        .iter()
        .map(Player::get_id)
        .collect::<Vec<_>>();
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[8])
        .expect("dq 8");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(players[7].get_id())
        .expect("dq 7");
    let (_w_bracket, l_bracket, _, _) = bracket.partition_matches().expect("enough players");
    assert!(
        l_bracket.iter().any(|m| {
            let Opponent(Some(auto)) = m.get_automatic_loser() else {
                return false;
            };
            auto == p[7]
        }),
        "p7 disqualified in losers"
    );
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(players[6].get_id())
        .expect("dq 6");
    let (_, l_bracket, _, _) = bracket.partition_matches().expect("enough players");
    assert_x_wins_against_y(&players[6], &players[7], &l_bracket);

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(players[5].get_id())
        .expect("dq 5");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(players[4].get_id())
        .expect("dq 4");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(players[3].get_id())
        .expect("dq 3");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(players[2].get_id())
        .expect("dq 2");

    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(players[1].get_id()),);
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(players[2].get_id()),);
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 1].get_players(),
        [Opponent(None), Opponent(None)],
        "expected no p in reset but got {:?}",
        bracket.get_matches()[bracket.get_matches().len() - 1].get_players()
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_automatic_loser(),
        Opponent(Some(players[2].get_id())),
        "expected automatic loser of grand finals to be {}",
        players[2]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_winner(),
        Opponent(Some(players[1].get_id())),
        "expected winner of grand finals to be {}\n{:?}",
        players[1],
        bracket.get_matches()[bracket.get_matches().len() - 2],
    );
    assert!(
        bracket.is_over(),
        "expected s to be over but got {bracket:?}"
    );
}

fn assert_x_wins_against_y(p1: &Player, p2: &Player, matches: &[Match]) {
    assert!(
        matches.iter().any(|m| {
            matches!((m.get_winner(), m.contains(p2.get_id())), (Opponent(Some(winner)), true) if winner == p1.get_id())
        }),
        "no matches where {} wins against {}",
        p1.get_name(),
        p2.get_name()
    );
}

#[test]
fn disqualify_from_winner() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("dq");
    let new_matches = bracket.matches_to_play();
    assert_eq!(
        new_matches.len(),
        1,
        "expected 1 match after DQ'ing p3 in 3 player tournament"
    );
    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), Score(2, 0), p[2].get_id())
        .expect("to report");
    assert_eq!(
        1,
        new_matches.len(),
        "expected 1 new match, see bracket {:?}",
        bracket
    );

    assert!(
        new_matches[0].contains(p[1].get_id()),
        "expected player 1 in GF"
    );
    assert!(
        new_matches[0].contains(p[2].get_id()),
        "expected player 2 in GF"
    );
}

#[test]
fn disqualify_in_double_elimination_bracket_from_loser() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );

    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), Score(2, 0), p[3].get_id())
        .expect("to report");
    assert_eq!(new_matches.len(), 1, "expected 1 new match");
    let (bracket, _, new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), Score(2, 0), p[2].get_id())
        .expect("to report");
    assert_eq!(new_matches.len(), 1, "expected 1 new match");

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("dq");
    let (_, _, gf, _) = bracket.partition_matches().expect("enough players");
    assert!(gf.contains(p[1].get_id()), "expected player 1 in GF");
    assert!(gf.contains(p[2].get_id()), "expected player 2 in GF");

    let new_matches = bracket.matches_to_play();
    assert_eq!(new_matches.len(), 1);

    assert!(
        new_matches[0].contains(p[1].get_id()),
        "expected player 1 in GF"
    );
    assert!(
        new_matches[0].contains(p[2].get_id()),
        "expected player 2 in GF"
    );
}

#[test]
fn disqualifying_everyone_in_double_elimination_tournament_is_imposible() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[8].get_id())
        .expect("dq 8");

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[7].get_id())
        .expect("dq 7");

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[6].get_id())
        .expect("dq 6");
    // R1 matches | R2 matches
    // 1-8
    //              1-?
    // 4-5
    // 2-7
    //              2-3
    // 3-6
    assert!(
        bracket.get_matches()[5].contains(p[2].get_id()),
        "expected {}",
        p[2]
    );
    assert!(
        bracket.get_matches()[5].contains(p[3].get_id()),
        "expected {}",
        p[3]
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[5].get_id())
        .expect("dq 5");
    // R1 matches | R2 matches
    // 1-8
    //              1-4
    // 4-5
    // 2-7
    //              2-3
    // 3-6
    assert!(
        bracket.get_matches()[4].contains(p[1].get_id()),
        "expected {} in new match after disqualifying {}",
        p[1],
        p[6]
    );
    assert!(
        bracket.get_matches()[4].contains(p[4].get_id()),
        "expected {} in new match after disqualifying {}",
        p[4],
        p[6]
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[4].get_id())
        .expect("dq 4");

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("dq 3");
    // p8--p3 DQ'ed
    // R1 matches | R2 matches | R3 matches
    // 1-8
    //              1-4
    // 4-5
    //                          1-2
    // 2-7
    //              2-3
    // 3-6
    //
    // Loser bracket
    // R1 matches | R2 matches | R3 matches | R4 matches
    // 5-8          4-5
    //                           3-4          ?-3
    // 6-7          3-6
    assert!(
        bracket.get_matches()[6].contains(p[1].get_id()),
        "expected {} in winner finals after disqualifying {}",
        p[1],
        p[3]
    );
    assert!(
        bracket.get_matches()[6].contains(p[2].get_id()),
        "expected {} in winner finals after disqualifying {}",
        p[2],
        p[6]
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("dq 2");
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[1].get_id()),);
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[2].get_id()),);
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_automatic_loser(),
        Opponent(Some(p[2].get_id())),
        "expected automatic loser of grand finals to be {}",
        p[2]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_winner(),
        Opponent(Some(p[1].get_id())),
        "expected winner of grand finals to be {}\n{:?}",
        p[1],
        bracket.get_matches()[bracket.get_matches().len() - 2],
    );
    assert!(
        bracket.is_over(),
        "expected bracket to be over but got {:?}",
        bracket.get_matches()
    );
}

#[test]
fn disqualifying_most_in_double_elimination_tournament_and_grand_finalist_from_winner_in_grand_finals(
) {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[8].get_id())
        .expect("dq 8");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[7].get_id())
        .expect("dq 7");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[6].get_id())
        .expect("dq 6");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[5].get_id())
        .expect("dq 5");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[4].get_id())
        .expect("dq 4");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("dq 3");
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), Score(2, 0), p[2].get_id())
        .expect("player 1 wins in winners finals");

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[1].get_id())
        .expect("dq 1");
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[1].get_id()),);
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[2].get_id()),);
    assert!(
        bracket.get_matches()[bracket.get_matches().len() - 1].contains(p[1].get_id()),
        "expected player 1 in reset",
    );
    assert!(bracket.get_matches()[bracket.get_matches().len() - 1].contains(p[2].get_id()),);
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_automatic_loser(),
        Opponent(Some(p[1].get_id())),
        "expected automatic loser of grand finals to be {}",
        p[2]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_winner(),
        Opponent(Some(p[2].get_id())),
        "expected winner of grand finals to be {}\n{:?}",
        p[1],
        bracket.get_matches()[bracket.get_matches().len() - 2]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 1].get_automatic_loser(),
        Opponent(Some(p[1].get_id())),
        "expected automatic loser of reset to be {}",
        p[1]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 1].get_winner(),
        Opponent(Some(p[2].get_id())),
        "expected winner of reset to be {}\n{:?}",
        p[2],
        bracket.get_matches()[bracket.get_matches().len() - 2],
    );
    assert!(
        bracket.is_over(),
        "expected bracket to be over but got {:?}",
        bracket.get_matches()
    );
}

#[test]
fn disqualifying_most_in_double_elimination_tournament_and_grand_finalist_from_loser_in_grand_finals(
) {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[8].get_id())
        .expect("dq 8");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[7].get_id())
        .expect("dq 7");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[6].get_id())
        .expect("dq 6");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[5].get_id())
        .expect("dq 5");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[4].get_id())
        .expect("dq 4");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("dq 3");
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[1].get_id(), Score(2, 0), p[2].get_id())
        .expect("player 1 wins in winners finals");

    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("dq 2");
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[1].get_id()),);
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[2].get_id()),);
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 1].get_players(),
        [Opponent(None), Opponent(None)]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_automatic_loser(),
        Opponent(Some(p[2].get_id())),
        "expected automatic loser of grand finals to be {}",
        p[2]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_winner(),
        Opponent(Some(p[1].get_id())),
        "expected winner of grand finals to be {}\n{:?}",
        p[1],
        bracket.get_matches()[bracket.get_matches().len() - 2],
    );
    assert!(
        bracket.is_over(),
        "expected bracket to be over but got {:?}",
        bracket.get_matches()
    );
}

#[test]
fn disqualifying_most_in_double_elimination_tournament_and_highest_expected_seed_in_winners_final()
{
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = Participants::default();
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding = seeding.add_participant(player).expect("seeding");
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding.get_player_list()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[8].get_id())
        .expect("dq 8");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[7].get_id())
        .expect("dq 7");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[6].get_id())
        .expect("dq 6");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[5].get_id())
        .expect("dq 5");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[4].get_id())
        .expect("dq 4");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("dq 3");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[1].get_id())
        .expect("dq 1");

    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[1].get_id()),);
    assert!(bracket.get_matches()[bracket.get_matches().len() - 2].contains(p[2].get_id()),);
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_players(),
        [Opponent(Some(p[2].get_id())), Opponent(Some(p[1].get_id()))],
        "expected player 1 and 2 in grand finals but got {:?}",
        bracket.get_matches()[bracket.get_matches().len() - 2].get_players()
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_automatic_loser(),
        Opponent(Some(p[1].get_id())),
        "expected automatic loser of grand finals to be {}",
        p[2]
    );
    assert_eq!(
        bracket.get_matches()[bracket.get_matches().len() - 2].get_winner(),
        Opponent(Some(p[2].get_id())),
        "expected winner of reset to be {}\n{:?}",
        p[2],
        bracket.get_matches()[bracket.get_matches().len() - 2],
    );
    assert!(
        bracket.is_over(),
        "expected bracket to be over but got {:?}",
        bracket.get_matches()
    );
}

#[test]
fn fuzzer_incident_01() {
    // DQ'ed player is in GF

    //     Win(false)
    // [1, 4] af286c3a-2be7-44ca-83ba-3b739f806aec
    //     Win(false)
    // [2, 3] 9d66fdf4-3c66-466f-9683-8e21357400fe
    //     Disqualification(false)
    // [1, 2] c469cb9c-d1f3-4acd-854d-e5a00a76a0d1
    //     Disqualification(false)
    // [3, 4] 3717b34e-fc15-42d6-89f1-8aaa64210593
    //     Disqualification(false) SKIPPED
    // [2, 3] 32d326ea-5be8-4baa-a28a-b238810dcd6d
    // Disqualification(false)
    // [1, 2] 2556a287-3a98-44e6-8ad1-42ab66baa723
    // * [1, 4] -p1 VS Wp4
    // * [2, 3] -p2 VS Wp3
    // * [1, 2] Wp4 VS Lp3
    // * [3, 4] Wp2 VS Lp1
    // * [2, 3] Lp3 VS Wp2
    // * [1, 2] -p4 VS -p2
    // * [1, 2] -?  VS -?
    let mut p = vec![PlayerID::create()];
    let mut seeding = vec![];
    for _ in 1..=4 {
        let id = PlayerID::create();
        p.push(id);
        seeding.push(id);
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[1], Score(0, 2), p[4])
        .expect("bracket");
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[2], Score(0, 2), p[3])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[1])
        .expect("bracket");
    let _bracket = bracket
        .disqualify_participant_from_bracket(p[2])
        .expect("bracket");
}

#[test]
fn fuzzer_incident_02() {
    // DQ'ed player is in loser bracket already

    // #total players: 8
    // #events: 16
    // Win(true)
    // Disqualification(true)
    // TOWin(true)
    // Disqualification(false)
    // Disqualification(false)
    // Disqualification(false)
    // Disqualification(false)
    // Disqualification(false) // SKIPPED
    // Disqualification(false) // SKIPPED
    // Disqualification(false) // SKIPPED
    // Disqualification(false) // SKIPPED
    // Disqualification(false) // PANIC
    // before crash:
    // * [1, 8] Wp1 VS -p8
    // * [2, 7] Lp2 VS Wp7
    // * [3, 6] Wp3 VS -p6
    // * [4, 5] Wp4 VS Lp5
    // * [1, 4] Wp1 VS Lp4
    // * [2, 3] Wp7 VS Lp3
    // * [1, 2] Wp1 VS Lp7
    // * [5, 8] Lp5 VS Wp8
    // * [6, 7] Wp6 VS Lp2
    // * [3, 6] Lp3 VS Wp6
    // * [4, 5] Lp4 VS Wp8
    // * [3, 4] -p6 VS -p8
    // * [2, 3] Lp7 VS -?
    // * [1, 2] -p1 VS -?
    // * [1, 2] -?  VS -?
    let mut p = vec![PlayerID::create()];
    let mut seeding = vec![];
    for _ in 1..=8 {
        let id = PlayerID::create();
        p.push(id);
        seeding.push(id);
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[1], Score(2, 0), p[8])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[2])
        .expect("bracket");
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[3], Score(2, 0), p[6])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[5])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[4])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[7])
        .expect("bracket");
    let _bracket = bracket
        .disqualify_participant_from_bracket(p[8])
        .expect("bracket");
}

#[test]
fn fuzzer_incident_03() {
    //match 0:TOWin(true)
    // 	* p1
    // 	* p2
    // 	* p3
    // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
    // 	* [1, 2] -p1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
    // 	* [2, 3] -?  VS -p3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
    // 	* [1, 2] -?  VS -?  | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
    // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
    // match 1:TOWin(true)

    // 	* p1
    // 	* p2
    // 	* p3
    // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
    // 	* [1, 2] Wp1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
    // 	* [2, 3] -p2 VS -p3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
    // 	* [1, 2] -p1 VS -?  | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
    // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
    // match 3:Disqualification(true)

    // 	* p1
    // 	* p2
    // 	* p3
    // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
    // 	* [1, 2] Wp1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
    // 	* [2, 3] -p2 VS -p3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
    // 	* [1, 2] Lp1 VS -?  | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
    // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
    // match 2:Disqualification(false)

    // 	* p1
    // 	* p2
    // 	* p3
    // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
    // 	* [1, 2] Wp1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
    // 	* [2, 3] Wp2 VS Lp3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
    // 	* [1, 2] Lp1 VS -p2 | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
    // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
    let mut p = vec![PlayerID::create()];
    let mut seeding = vec![];
    for _ in 1..=3 {
        let id = PlayerID::create();
        p.push(id);
        seeding.push(id);
    }
    let bracket = DoubleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
        AutomaticMatchValidationMode::Flexible,
        MatchFormat::ft3(),
        None,
    );
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[2], Score(2, 0), p[3])
        .expect("bracket");
    let (bracket, _, _) = bracket
        .tournament_organiser_reports_result_dangerous(p[1], Score(2, 0), p[2])
        .expect("bracket");
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[1])
        .expect("bracket");
    // when moved into grand finals with someone disqualified, match should
    // be updated
    let (bracket, _) = bracket
        .disqualify_participant_from_bracket(p[3])
        .expect("bracket");
    assert!(bracket.is_over());
}
