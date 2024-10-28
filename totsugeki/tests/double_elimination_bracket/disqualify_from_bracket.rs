use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::matches::{partition_double_elimination_matches, Match};
use totsugeki::opponent::Opponent;
use totsugeki::player::{Participants, Player};
use totsugeki::validation::AutomaticMatchValidationMode;
use totsugeki::{bracket, ID};
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
    let (winners, losers, _, _) =
        partition_double_elimination_matches(&bracket.get_matches(), bracket.get_seeding().len());
    assert!(
        !winners.iter().any(|m| m.contains(p[n].get_id())
            && m.get_winner() == Opponent::Unknown
            && m.get_automatic_loser() == Opponent::Unknown),
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
        Seeding::new(participants.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Flexible,
    );

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id())
        ),
        "expected player 1 not to be declared looser in any match"
    );
    let bracket = bracket
        .disqualify_participant_from_bracket(p[1].get_id())
        .expect("bracket with player 1 disqualified");
    assert!(
        bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id())
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
        Seeding::new(participants.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Strict,
    );

    let (bracket, match_id_p2, _new_matches) = bracket
        .tournament_organiser_reports_result_dangerous(p[2].get_id(), (2, 0), p[3].get_id())
        .expect("reported result by player 2");
    let (bracket, _) = bracket.validate_match_result(match_id_p2);

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
        ),
        "expected player 2 not to be declared looser in any match"
    );
    let bracket = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("p2 is disqualified");

    let p1_wins_against_p2_since_p2_is_disqualified =
        bracket
            .get_matches()
            .iter()
            .any(|m| match (m.get_automatic_loser(), m.get_winner()) {
                (Opponent::Player(loser), Opponent::Player(winner)) if loser == p[2].get_id() => {
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
        Seeding::new(participants.get_seeding()).unwrap(),
        AutomaticMatchValidationMode::Strict,
    );

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
        ),
        "expected player 2 not to be declared looser in any match"
    );
    let bracket = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("bracket with player 2 disqualified");
    assert!(
        bracket
            .get_matches()
            .iter()
            .any(|m| matches!(m.get_automatic_loser(), Opponent::Player(player) if player == p[2].get_id())),
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
        DoubleEliminationBracket::create(Seeding::new(participants.get_seeding()).unwrap(), auto),
        p,
    )
}

fn assert_outcome(bracket: &DoubleEliminationBracket, x: &Player, y: &Player) {
    assert!(
        bracket.get_matches().iter().any(|m| matches!((
                m.contains(x.get_id()),
                m.contains(y.get_id()),
                m.get_winner()
            ), (true, true, Opponent::Player(winner)) if winner == x.get_id())),
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
            ), (true, true, Opponent::Player(winner)) if winner == x.get_id())),
        "No match where {} wins against {}",
        x.get_name(),
        y.get_name()
    );
}

#[test]
fn disqualifying_everyone_is_impossible_because_the_last_player_remaining_wins_grand_finals_automatically(
) {
    let auto = true;
    let (bracket, p) = initial_step(8, AutomaticMatchValidationMode::Flexible);

    let bracket = bracket
        .disqualify_participant_from_bracket(p[2].get_id())
        .expect("p2 DQ'ed");
    assert_player_drops_to_losers(&bracket, 2, &p);
    assert_outcome(&bracket, &p[7], &p[2]);

    let bracket = bracket
        .disqualify_participant_from_bracket(p[3].get_id())
        .expect("p3 DQ'ed");
    assert_player_drops_to_losers(&bracket, 3, &p);
    assert_outcome(&bracket, &p[6], &p[3]);

    let bracket = bracket
        .disqualify_participant_from_bracket(p[4].get_id())
        .expect("p4 DQ'ed");
    assert_outcome(&bracket, &p[5], &p[4]);
    assert_player_drops_to_losers(&bracket, 4, &p);
    let (_, l_bracket, _, _) =
        partition_double_elimination_matches(&bracket.get_matches(), bracket.get_seeding().len());
    assert_eq!(
        l_bracket
            .iter()
            .filter(|m| m.contains(p[4].get_id()))
            .count(),
        1
    );

    let bracket = bracket
        .disqualify_participant_from_bracket(p[5].get_id())
        .expect("p5 DQ'ed");
    // player 5 opponent in winners is unknown yet he can drop to losers
    // already, even if 1vs8 has not been played out
    assert_player_drops_to_losers(&bracket, 5, &p);

    let bracket = bracket
        .disqualify_participant_from_bracket(p[6].get_id())
        .expect("p6 DQ'ed");
    assert_outcome(&bracket, &p[7], &p[6]);

    let bracket = bracket
        .disqualify_participant_from_bracket(p[7].get_id())
        .expect("p7 DQ'ed");
    assert_player_drops_to_losers(&bracket, 7, &p);
    assert!(&bracket
        .get_matches()
        .iter()
        .any(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3]));
    let (_w_bracket, l_bracket, _, _) =
        partition_double_elimination_matches(&bracket.get_matches(), bracket.get_seeding().len());
    let m = &l_bracket
        .iter()
        .find(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3])
        .expect("m");
    let Opponent::Player(loser) = m.get_automatic_loser() else {
        panic!("expected loser but found none {m:?}");
    };
    assert_eq!(loser, p[7].get_id());

    let bracket = bracket
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
        partition_double_elimination_matches(&bracket.get_matches(), bracket.get_seeding().len());
    for m in &winner_bracket {
        assert_ne!(
            m.get_automatic_loser(),
            Opponent::Unknown,
            "expected winner bracket match to have automatic loser but got none: {m:?}"
        );
    }
    for m in &loser_bracket {
        assert_ne!(
            m.get_automatic_loser(),
            Opponent::Unknown,
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
