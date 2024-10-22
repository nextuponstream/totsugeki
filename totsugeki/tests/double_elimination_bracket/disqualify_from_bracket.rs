use totsugeki::bracket::seeding::Seeding;
use totsugeki::double_elimination_bracket::progression::ProgressionDEB;
use totsugeki::double_elimination_bracket::DoubleEliminationBracket;
use totsugeki::opponent::Opponent;
use totsugeki::player::{Participants, Player};
use totsugeki::validation::AutomaticMatchValidationMode;
use totsugeki::ID;
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
