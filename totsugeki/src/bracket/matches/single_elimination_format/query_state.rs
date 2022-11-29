//! Query state of single elimination bracket

#[cfg(test)]
mod tests {

    use crate::{
        bracket::matches::{
            assert_elimination, assert_next_matches, single_elimination_format::Step, Progression,
        },
        player::{Participants, Player},
    };

    #[test]
    fn run_5_man_bracket() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("new player");
        }
        let auto = false;
        let s = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("seeding");

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1, "{new_matches:?}");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_next_matches(&s, &[], &[(1, 5), (2, 3)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 1), p[5].get_id())
            .expect("winner 1vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1], &[(2, 3)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs3");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 1);
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_next_matches(&s, &[], &[(1, 3)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs3");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        assert_eq!(new_matches.len(), 0);

        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("s");
        assert_elimination(&s, &p, 3);
    }

    #[test]
    fn run_5_man_bracket_automated() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("new player");
        }
        let auto = true;
        let s = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("seeding");

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1, "{new_matches:?}");
        assert_next_matches(&s, &[], &[(1, 5), (2, 3)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 1), p[5].get_id())
            .expect("winner 1vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1], &[(2, 3)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs3");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(1, 3)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs3");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&s, &p, 3);
    }

    #[test]
    fn bracket_8_man() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("new player");
        }
        let auto = false;

        let s = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("seeding");

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
            .expect("winner 1vs8");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1], &[(2, 7), (3, 6), (4, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
            .expect("winner 2vs7");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1, 2], &[(3, 6), (4, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[2], &[(3, 6), (1, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[2, 5], &[(3, 6)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[3].get_id())
            .expect("winner 3vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[5], &[(2, 6)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(5, 6)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[6].get_id())
            .expect("winner 5vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&s, &p, 5);
    }

    #[test]
    fn bracket_8_man_automated() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("new player");
        }
        let auto = true;
        let s = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("seeding");

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
            .expect("winner 1vs8");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1], &[(2, 7), (3, 6), (4, 5)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
            .expect("winner 2vs7");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1, 2], &[(3, 6), (4, 5)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[2], &[(3, 6), (1, 5)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[2, 5], &[(3, 6)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[3].get_id())
            .expect("winner 3vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[5], &[(2, 6)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[2].get_id())
            .expect("winner 2vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(5, 6)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[6].get_id())
            .expect("winner 5vs6");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("s");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&s, &p, 5);
    }

    #[test]
    fn bracket_9_man() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=9 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("new player");
        }
        let auto = false;
        let s = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("seeding");

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[8].get_id())
            .expect("winner 8vs9");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[5], &[(1, 9), (3, 6), (2, 7)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("winner 3vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[3, 5], &[(1, 9), (2, 7)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[7].get_id(), (2, 0), p[2].get_id())
            .expect("winner 3vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[5], &[(1, 9), (3, 7)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[7].get_id())
            .expect("winner 3vs7");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[3, 5], &[(1, 9)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs9");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[3], &[(9, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[5].get_id())
            .expect("winner 5vs9");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(3, 9)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[9].get_id())
            .expect("winner 3vs9");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&s, &p, 3);
    }

    #[test]
    fn bracket_9_man_automated() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=9 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("new player");
        }
        let auto = true;
        let s = Step::new(
            None,
            seeding.clone(),
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("seeding");

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("winner 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[1, 5], &[(8, 9), (3, 6), (2, 7)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[8].get_id())
            .expect("winner 8vs9");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[5], &[(1, 9), (3, 6), (2, 7)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("winner 3vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[3, 5], &[(1, 9), (2, 7)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[7].get_id(), (2, 0), p[2].get_id())
            .expect("winner 3vs6");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[5], &[(1, 9), (3, 7)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[7].get_id())
            .expect("winner 3vs7");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[3, 5], &[(1, 9)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[1].get_id())
            .expect("winner 1vs9");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[3], &[(9, 5)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[9].get_id(), (2, 0), p[5].get_id())
            .expect("winner 5vs9");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(3, 9)], &p);

        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[9].get_id())
            .expect("winner 3vs9");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&s, &p, 3);
    }
}
