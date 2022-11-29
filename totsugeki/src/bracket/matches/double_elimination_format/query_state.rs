//! tests for double elimination bracket

#[cfg(test)]
mod tests {
    use crate::{
        bracket::matches::{
            assert_elimination, assert_next_matches, double_elimination_format::Step, Progression,
        },
        player::{Participants, Player},
    };

    #[test]
    fn bracket_5_man_with_frequent_upsets() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
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
        .expect("step");

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
            .expect("winner 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[5], &[(1, 4), (2, 3)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[4].get_id())
            .expect("winner 1vs4");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[4], &[(2, 3), (1, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("winner 2vs3");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[3], &[(1, 5), (2, 4)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[5].get_id())
            .expect("loser 1vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(2, 4), (3, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[4].get_id())
            .expect("winner 2vs4");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert_next_matches(&s, &[2, 4], &[(3, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[3].get_id(), (0, 2), p[5].get_id())
            .expect("loser 3vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[2], &[(4, 5)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
            .expect("loser 4vs5");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(2, 4)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[4].get_id())
            .expect("grand finals 2vs4");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1);
        assert_next_matches(&s, &[], &[(2, 4)], &p);

        let (matches, match_id, _) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[4].get_id())
            .expect("reset 2vs4");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, new_matches) = s.validate_match_result(match_id).expect("validation");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);

        assert_elimination(&s, &p, 2);
    }
}
