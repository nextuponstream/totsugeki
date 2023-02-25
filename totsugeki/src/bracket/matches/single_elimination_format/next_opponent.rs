//! test next opponent functionnality for single elimination bracket

#[cfg(test)]
mod tests {
    use crate::{
        bracket::matches::{single_elimination_format::Step, Progression},
        opponent::Opponent,
        player::{Participants, Player},
    };

    fn assert_players_play_each_other(
        player_1: usize,
        player_2: usize,
        player_ids: &[Player],
        s: &dyn Progression,
    ) {
        let (next_opponent, match_id_1) = s
            .next_opponent(player_ids[player_1].get_id())
            .expect("next opponent");
        let Opponent::Player(next_opponent) = next_opponent else {
            panic!("expected player");
        };
        assert_eq!(next_opponent, player_ids[player_2].get_id());

        let (next_opponent, match_id_2) = s
            .next_opponent(player_ids[player_2].get_id())
            .expect("next opponent");
        let Opponent::Player(next_opponent) = next_opponent else {
            panic!("expected player")
        };
        assert_eq!(next_opponent, player_ids[player_1].get_id());

        assert_eq!(
            match_id_1, match_id_2,
            "expected player to be playing the same match"
        );
    }

    #[test]
    fn run_3_man() {
        let mut p = vec![Player::new("don't use".into())]; // padding for readability
        let mut bad_seeding = Participants::default();
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
            bad_seeding = bad_seeding.add_participant(player).expect("bracket");
        }
        let auto = true;
        let s = Step::new(None, &seeding, auto).expect("step");

        assert_eq!(s.matches.len(), 2);
        assert_eq!(s.matches_to_play().len(), 1);
        assert_players_play_each_other(2, 3, &p, &s);
        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("bracket");
        let s = Step::new(Some(matches), &s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1, "grand finals match generated");
        assert_players_play_each_other(1, 2, &p, &s);
        assert_eq!(s.matches_to_play().len(), 1);
        let (matches, _, new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[2].get_id())
            .expect("bracket");
        let s = Step::new(Some(matches), &s.seeding, auto).expect("step");
        assert!(s.matches_to_play().is_empty());
        assert!(new_matches.is_empty());
        assert!(s.is_over());
    }

    #[test]
    fn run_5_man_bracket() {
        let mut p = vec![Player::new("don't use".into())]; // padding for readability
        let mut bad_seeding = Participants::default();
        let mut seeding = vec![];
        for i in 1..=5 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
            bad_seeding = bad_seeding.add_participant(player).expect("bracket");
        }
        let auto = true;
        let s = Step::new(None, &seeding, auto).expect("step");

        assert_eq!(s.matches.len(), 4);
        assert_eq!(s.matches_to_play().len(), 2);
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
            .expect("bracket");
        let s = Step::new(Some(matches), &s.seeding, auto).expect("step");
        assert_eq!(s.matches_to_play().len(), 2);
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
            .expect("bracket");
        let s = Step::new(Some(matches), &s.seeding, auto).expect("step");
        assert_eq!(s.matches_to_play().len(), 1);
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[4].get_id())
            .expect("bracket");
        let s = Step::new(Some(matches), &s.seeding, auto).expect("step");
        assert_eq!(s.matches_to_play().len(), 1);
        let (matches, _, _new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[3].get_id())
            .expect("bracket");
        let s = Step::new(Some(matches), &s.seeding, auto).expect("step");
        assert!(s.is_over());
        assert_eq!(s.matches_to_play().len(), 0);
    }
}
