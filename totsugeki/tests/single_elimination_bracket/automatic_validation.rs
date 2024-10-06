use crate::single_elimination_bracket::{
    assert_next_matches, assert_no_next_match_after_tournament_is_over,
};
use totsugeki::bracket::seeding::Seeding;
use totsugeki::next_opponent::NextOpponentInBracket;
use totsugeki::opponent::Opponent;
use totsugeki::player::{Participants, Player};
use totsugeki::single_elimination_bracket::progression::ProgressionSEB;
use totsugeki::single_elimination_bracket::SingleEliminationBracket;
use totsugeki::ID;

// FIXME duplicate definition
fn assert_players_play_each_other(
    player_1: usize,
    player_2: usize,
    player_ids: &[Player],
    bracket: &SingleEliminationBracket,
) {
    let Some((Some(Opponent::Player(next_opponent)), match_id_1)) =
        bracket.next_opponent_in_bracket(player_ids[player_1].get_id())
    else {
        panic!("No next opponent")
    };
    assert_eq!(next_opponent, player_ids[player_2].get_id());

    let Some((Some(Opponent::Player(next_opponent)), match_id_2)) =
        bracket.next_opponent_in_bracket(player_ids[player_2].get_id())
    else {
        panic!("No next opponent")
    };
    assert_eq!(next_opponent, player_ids[player_1].get_id());

    assert_eq!(
        match_id_1, match_id_2,
        "expected player to be playing the same match"
    );
}

// FIXME duplicate definition
fn assert_players_play_each_other_ids(
    player_1: usize,
    player_2: usize,
    player_ids: &[ID],
    bracket: &SingleEliminationBracket,
) {
    let (next_opponent, match_id_1) = bracket
        .next_opponent(player_ids[player_1])
        .expect("next opponent");
    let Opponent::Player(next_opponent) = next_opponent else {
        panic!("expected player")
    };
    assert_eq!(next_opponent, player_ids[player_2]);
    let (next_opponent, match_id_2) = bracket
        .next_opponent(player_ids[player_2])
        .expect("next opponent");
    let Opponent::Player(next_opponent) = next_opponent else {
        panic!("expected player")
    };
    assert_eq!(next_opponent, player_ids[player_1]);

    assert_eq!(
        match_id_1, match_id_2,
        "expected player to be playing the same match"
    );
}

mod player_report_before_organiser {
    use super::*;

    #[test]
    fn higher_seed_reports_before_to() {
        let mut player_ids = vec![ID::new_v4()]; // padding for readability
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding.push(player.get_id());
        }

        // player 2 reports before TO does
        let bracket =
            SingleEliminationBracket::create(Seeding::new(seeding.clone()).unwrap(), true);
        assert_players_play_each_other_ids(2, 3, &player_ids, &bracket);
        let (bracket, _, _) = bracket
            .report_result(player_ids[2], (2, 0))
            .expect("bracket");
        let (_, _, _) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
    }
    #[test]
    fn lower_seed_reports_before_to() {
        let mut player_ids = vec![ID::new_v4()]; // padding for readability
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding.push(player.get_id());
        }

        // player 3 reports before TO does
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);
        assert_players_play_each_other_ids(2, 3, &player_ids, &bracket);
        let (bracket, _, _) = bracket
            .report_result(player_ids[3], (0, 2))
            .expect("bracket");
        let (_, _, _) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("bracket");
    }
}

mod bracket_with_3_participants {
    use super::*;
    #[test]
    fn example1() {
        let mut p = vec![Player::new("don't use".into())]; // padding for readability
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
        }
        let seb = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        assert_eq!(seb.get_matches().len(), 2);
        assert_eq!(seb.matches_to_play().len(), 1);
        assert_players_play_each_other(2, 3, &p, &seb);
        let (bracket, _, new_matches) = seb
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("bracket");
        assert_eq!(new_matches.len(), 1, "grand finals match generated");
        assert_players_play_each_other(1, 2, &p, &bracket);
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[2].get_id())
            .expect("bracket");
        assert!(bracket.matches_to_play().is_empty());
        assert!(new_matches.is_empty());
        assert!(bracket.is_over());
    }

    #[test]
    fn example2() {
        let mut seeding = Participants::default();
        let mut player_ids = vec![ID::new_v4()]; // padding
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            seeding = seeding.add_participant(player).expect("updated seeding");
        }
        let seeding = seeding
            .get_players_list()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<_>>();
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        assert_eq!(bracket.get_matches().len(), 2);
        assert_eq!(bracket.matches_to_play().len(), 1);
        assert_players_play_each_other_ids(2, 3, &player_ids, &bracket);
        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(player_ids[2], (2, 0), player_ids[3])
            .expect("matches");
        assert_ne!(bracket.get_matches()[0].get_winner(), Opponent::Unknown);
        assert_eq!(new_matches.len(), 1, "grand finals match generated");
        assert_players_play_each_other_ids(1, 2, &player_ids, &bracket);
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(player_ids[1], (0, 2), player_ids[2])
            .expect("matches");
        assert!(bracket.matches_to_play().is_empty());
        assert!(new_matches.is_empty());
        assert!(bracket.is_over());
    }
}

mod bracket_with_5_participants {
    use super::*;
    #[test]
    fn example1() {
        let mut p = vec![Player::new("don't use".into())]; // padding for readability
        let mut seeding = vec![];
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
        }
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        assert_eq!(bracket.get_matches().len(), 4);
        assert_eq!(bracket.matches_to_play().len(), 2);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
            .expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 2);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
            .expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[4].get_id())
            .expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[3].get_id())
            .expect("bracket");
        assert!(bracket.is_over());
        assert_eq!(bracket.matches_to_play().len(), 0);
    }

    #[test]
    fn example2() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = vec![];
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
        }
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 1, "{new_matches:?}");
        assert_next_matches(&bracket, &[], &[(1, 5), (2, 3)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), (2, 1), p[5].get_id())
            .expect("winner 1vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 3)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs3");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(1, 3)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs3");
        assert_eq!(new_matches.len(), 0);

        assert_no_next_match_after_tournament_is_over(&bracket);
    }

    #[test]
    fn example3() {
        let mut bad_seeding = Participants::default();
        let mut player_ids = vec![ID::new_v4()]; // padding
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            bad_seeding = bad_seeding
                .add_participant(player)
                .expect("updated seeding");
        }
        let seeding = bad_seeding
            .get_players_list()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<_>>();

        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);
        assert_eq!(bracket.get_matches().len(), 4);
        assert_eq!(bracket.matches_to_play().len(), 2);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(player_ids[4], (2, 0), player_ids[5])
            .expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 2);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(player_ids[2], (0, 2), player_ids[3])
            .expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[4])
            .expect("bracket");
        assert_eq!(bracket.matches_to_play().len(), 1);
        let (bracket, _, _new_matches) = bracket
            .tournament_organiser_reports_result(player_ids[1], (2, 0), player_ids[3])
            .expect("bracket");
        if !bracket.is_over() {
            for m in bracket.get_matches() {
                println!("{m}");
            }
            panic!("expected bracket to be over")
        }
        assert_eq!(bracket.matches_to_play().len(), 0);
    }

    #[test]
    fn example4() {
        let mut p = vec![Player::new("don't use".into())];
        let mut bad_seeding = Participants::default();
        let mut seeding = vec![];
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
        }
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 1, "{new_matches:?}");
        assert_next_matches(&bracket, &[], &[(1, 5), (2, 3)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), (2, 1), p[5].get_id())
            .expect("winner 1vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 3)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs3");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(1, 3)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs3");
        assert_eq!(new_matches.len(), 0);

        assert_no_next_match_after_tournament_is_over(&bracket);
    }
}

mod bracket_with_8_participants {
    use super::*;

    #[test]
    fn example1() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = vec![];
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
        }
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
            .expect("winner 1vs8");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 7), (3, 6), (4, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
            .expect("winner 2vs7");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 2], &[(3, 6), (4, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[2], &[(3, 6), (1, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[2, 5], &[(3, 6)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[3].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(2, 6)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(5, 6)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[6].get_id())
            .expect("winner 5vs6");
        assert_eq!(new_matches.len(), 0);

        assert_no_next_match_after_tournament_is_over(&bracket);
    }

    #[test]
    fn example2() {
        let mut p = vec![Player::new("don't use".into())];
        let mut bad_seeding = Participants::default();
        let mut seeding = vec![];
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
            bad_seeding = bad_seeding.add_participant(player).expect("new player");
        }
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
            .expect("winner 1vs8");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1], &[(2, 7), (3, 6), (4, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
            .expect("winner 2vs7");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 2], &[(3, 6), (4, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[2], &[(3, 6), (1, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[2, 5], &[(3, 6)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[3].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(2, 6)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(5, 6)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[6].get_id())
            .expect("winner 5vs6");
        assert_eq!(new_matches.len(), 0);

        assert_no_next_match_after_tournament_is_over(&bracket);
    }
}

mod bracket_with_9_participants {
    use super::*;
    use totsugeki::seeding::seed;

    #[test]
    fn example1() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = vec![];
        for i in 1..=9 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
        }
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[8].get_id())
            .expect("winner 8vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 6), (2, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9), (2, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[7].get_id(), (2, 0), p[2].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[7].get_id())
            .expect("winner 3vs7");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[3], &[(9, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[5].get_id())
            .expect("winner 5vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(3, 9)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[9].get_id())
            .expect("winner 3vs9");
        assert_eq!(new_matches.len(), 0);

        assert_no_next_match_after_tournament_is_over(&bracket);
    }

    #[test]
    fn example2() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = vec![];
        for i in 1..=9 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
        }
        let bracket = SingleEliminationBracket::create(Seeding::new(seeding).unwrap(), true);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[8].get_id())
            .expect("winner 8vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 6), (2, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9), (2, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[7].get_id(), (2, 0), p[2].get_id())
            .expect("winner 3vs6");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[5], &[(1, 9), (3, 7)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[7].get_id())
            .expect("winner 3vs7");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&bracket, &[3, 5], &[(1, 9)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[3], &[(9, 5)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[5].get_id())
            .expect("winner 5vs9");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&bracket, &[], &[(3, 9)], &p);

        let (bracket, _, new_matches) = bracket
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[9].get_id())
            .expect("winner 3vs9");
        assert_eq!(new_matches.len(), 0);

        assert_no_next_match_after_tournament_is_over(&bracket);
    }
}
