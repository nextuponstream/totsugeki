//! test next opponent functionnality for double elimination implementation

#[cfg(test)]
mod tests {

    use crate::{
        bracket::matches::{double_elimination_format::Step, Progression},
        player::{Participants, Player},
    };

    #[test]
    fn bracket_run_3_man() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
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
        .expect("step");

        assert_eq!(s.matches.len(), 5);
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[2].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[3].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert!(s.is_over());
    }

    #[test]
    fn run_5_man() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
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
        .expect("step");

        assert_eq!(s.matches.len(), 9);
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
            .expect("step");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[4].get_id(), (0, 2), p[5].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[5].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[3].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[4].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[5].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[1].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert!(s.is_over());
    }

    #[test]
    fn run_8_no_upsets() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
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
        .expect("step");

        assert_eq!(s.matches.len(), 15);
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[7].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[5].get_id(), (2, 0), p[8].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[6].get_id(), (2, 0), p[7].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[4].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[4].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("s");

        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert!(s.is_over());
    }

    #[test]
    fn run_8_man_bracket_with_frequent_upsets() {
        // every 2 matches, there is an upset
        let mut p = vec![Player::new("don't use".into())];
        let mut unpadded_p = vec![];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            unpadded_p.push(player.id);
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = false;
        let s = Step::new(None, seeding.clone(), unpadded_p, auto).expect("step");
        assert_eq!(s.matches.len(), 15);

        let (matches, winner_1vs8, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[8].get_id())
            .expect("bracket");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(winner_1vs8).expect("bracket");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, winner_2vs7, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[7].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(winner_2vs7).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, winner_3vs6, _new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(winner_3vs6).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, winner_4vs5, _new_matches) = s
            .tournament_organiser_reports_result(p[4].get_id(), (0, 2), p[5].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(winner_4vs5).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, loser_4vs8, _new_matches) = s
            .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[8].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(loser_4vs8).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, loser_2vs6, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[6].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(loser_2vs6).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, winner_1vs5, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[5].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(winner_1vs5).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, winner_3vs7, _new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (0, 2), p[7].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(winner_3vs7).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, loser_3vs6, _new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[6].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(loser_3vs6).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, loser_4vs5, _new_matches) = s
            .tournament_organiser_reports_result(p[4].get_id(), (0, 2), p[5].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(loser_4vs5).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, loser_3vs5, _new_matches) = s
            .tournament_organiser_reports_result(p[3].get_id(), (2, 0), p[5].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(loser_3vs5).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, winner_1vs7, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[7].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(winner_1vs7).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, loser_1vs3, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[3].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(loser_1vs3).expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, grand_finals, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[7].get_id())
            .expect("s");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s.validate_match_result(grand_finals).expect("bracket");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert!(s.is_over(), "{s:?}");
    }
}
