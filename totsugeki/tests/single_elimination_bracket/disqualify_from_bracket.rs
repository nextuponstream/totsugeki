use crate::common::assert_outcome;
use totsugeki::bracket::seeding::Seeding;
use totsugeki::opponent::Opponent;
use totsugeki::player::Player;
use totsugeki::single_elimination_bracket::progression::ProgressionSEB;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::ID;

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
    let auto = false;
    let bracket = SingleEliminationBracket::create(seeding, auto);
    let bracket = bracket.disqualify_participant_from_bracket(p[2].get_id());
    assert_outcome(&bracket.get_matches(), &p[7], &p[2]);
    let bracket = bracket.disqualify_participant_from_bracket(p[3].get_id());
    assert_outcome(&bracket.get_matches(), &p[6], &p[3]);
    let bracket = bracket.disqualify_participant_from_bracket(p[4].get_id());
    assert_outcome(&bracket.get_matches(), &p[5], &p[4]);
    let bracket = bracket.disqualify_participant_from_bracket(p[5].get_id());
    // player 5 opponent is unknown
    let bracket = bracket.disqualify_participant_from_bracket(p[6].get_id());
    assert_outcome(&bracket.get_matches(), &p[7], &p[6]);
    let bracket = bracket.disqualify_participant_from_bracket(p[7].get_id());
    // player 7 is in GF
    let bracket = bracket.disqualify_participant_from_bracket(p[8].get_id());
    assert_outcome(&bracket.get_matches(), &p[1], &p[8]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[5]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[7]);

    let bracket = bracket.disqualify_participant_from_bracket(p[1].get_id());
    assert_outcome(&bracket.get_matches(), &p[1], &p[8]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[5]);
    assert_outcome(&bracket.get_matches(), &p[1], &p[7]);
}

#[test]
fn disqualifying_unknown_player_is_a_no_op() {
    let mut seeding = vec![];
    for i in 1..=3 {
        seeding.push(ID::new_v4())
    }
    let seeding = Seeding::new(seeding).unwrap();
    let bracket = SingleEliminationBracket::create(seeding, false);

    let unknown_player = ID::new_v4();
    bracket.disqualify_participant_from_bracket(unknown_player);
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
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id())
        ),
        "expected player 1 not to be declared looser in any match"
    );
    let bracket = bracket.disqualify_participant_from_bracket(p[1].get_id());
    assert!(
        bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id())
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
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), false);

    let (bracket, match_id_p2, _new_matches) = bracket
        .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
        .unwrap();
    let (bracket, _) = bracket.validate_match_result(match_id_p2);

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
        ),
        "expected player 2 not to be declared looser in any match"
    );
    let bracket = bracket.disqualify_participant_from_bracket(p[2].get_id());
    assert!(
        bracket.get_matches().iter().any(|m| matches!(
                    (m.get_automatic_loser(), m.get_winner()),
                    (Opponent::Player(loser), Opponent::Player(winner)) 
                    if loser == p[2].get_id() && winner == p[1].get_id())),
        "expected player 1 winning match where player 2 is disqualified, got {:?}",
        bracket.get_matches()
    );
    assert!(
        bracket
            .get_matches()
            .iter()
            .all(|m| m.get_winner() != Opponent::Unknown),
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
    let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), false);

    assert!(
        !bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
        ),
        "expected player 2 not to be declared looser in any match"
    );
    let bracket = bracket.disqualify_participant_from_bracket(p[2].get_id());
    assert!(
        bracket.get_matches().iter().any(
            |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
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
