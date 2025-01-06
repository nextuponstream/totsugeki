use crate::common::assert_outcome;
use totsugeki::bracket::seeding::Seeding;
use totsugeki::matches::result::{MatchFormat, Score};
use totsugeki::opponent::Opponent;
use totsugeki::player::{Player, PlayerID};
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::validation::AutomaticMatchValidationMode;

#[test]
fn disqualifying_everyone() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=8 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let seeding = Seeding::new(seeding).unwrap();
    let auto = AutomaticMatchValidationMode::Strict;
    let bracket = SingleEliminationBracket::create(seeding, auto, MatchFormat::ft3(), None);
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[2].get_id());
    assert_outcome(&bracket.get_matches(), &p[7], &p[2]);
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[3].get_id());
    assert_outcome(&bracket.get_matches(), &p[6], &p[3]);
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[4].get_id());
    assert_outcome(&bracket.get_matches(), &p[5], &p[4]);
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[5].get_id());
    // player 5 opponent is unknown
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[6].get_id());
    assert_outcome(&bracket.get_matches(), &p[7], &p[6]);
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[7].get_id());
    // player 7 is in GF
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[8].get_id());
    assert_outcome(&bracket.get_matches(), &p[1], &p[8]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[5]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[7]);

    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[1].get_id());
    assert_outcome(&bracket.get_matches(), &p[1], &p[8]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[5]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[7]);
}

#[test]
fn disqualifying_unknown_player_is_a_no_op() {
    let mut seeding = vec![];
    for _ in 1..=3 {
        seeding.push(PlayerID::create())
    }
    let seeding = Seeding::new(seeding).unwrap();
    let bracket = SingleEliminationBracket::create(
        seeding,
        AutomaticMatchValidationMode::Strict,
        MatchFormat::ft3(),
        None,
    );

    let unknown_player = PlayerID::create();
    let _ = bracket.disqualify_participant_from_bracket(unknown_player);
}

#[test]
fn opponent_of_disqualified_player_can_play_their_next_match() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
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
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[1].get_id());
    assert!(
        bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(loser)) if loser == p[1].get_id())
        ),
        "expected match where player 1 is declared loser"
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
    let mut seeding = vec![];
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
        AutomaticMatchValidationMode::Strict,
        MatchFormat::ft3(),
        None,
    );

    let (bracket, match_id_p2, _new_matches) = bracket
        .tournament_organiser_reports_result(p[2].get_id(), Score(2, 0), p[3].get_id())
        .unwrap();
    let (bracket, _) = bracket.validate_match_result(match_id_p2);

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(loser)) if loser == p[2].get_id())
        ),
        "expected player 2 not to be declared looser in any match"
    );
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[2].get_id());
    assert!(
        bracket.get_matches().iter().any(|m| matches!(
                    (m.get_automatic_loser(), m.get_winner()),
                    (Opponent(Some(loser)), Opponent(Some(winner))) 
                    if loser == p[2].get_id() && winner == p[1].get_id())),
        "expected player 1 winning match where player 2 is disqualified, got {:?}",
        bracket.get_matches()
    );
    assert!(
        bracket
            .get_matches()
            .iter()
            .all(|m| m.get_winner() != Opponent(None)),
        "expected all matches were played"
    );
}

#[test]
fn disqualifying_player_sets_their_opponent_as_the_winner_and_they_move_to_their_next_match() {
    let mut p = vec![Player::new("don't use".into())];
    let mut seeding = vec![];
    for i in 1..=3 {
        let player = Player::new(format!("p{i}"));
        p.push(player.clone());
        seeding.push(player.get_id());
    }
    let bracket = SingleEliminationBracket::create(
        Seeding::new(seeding).unwrap(),
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
    let (bracket, _new_playable_matches) =
        bracket.disqualify_participant_from_bracket(p[2].get_id());
    assert!(
        bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent(Some(loser)) if loser == p[2].get_id())
        ),
        "expected match where player 2 is declared looser"
    );
    assert!(
        bracket
            .get_matches()
            .iter()
            .any(|m| m.contains(p[1].get_id()) && m.contains(p[3].get_id())),
        "expected player 1 and 3 playing in grand finals"
    );
}
