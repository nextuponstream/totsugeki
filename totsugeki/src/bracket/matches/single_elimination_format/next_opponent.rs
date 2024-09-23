//! test next opponent functionnality for single elimination bracket

#[cfg(test)]
mod tests {
    use crate::bracket::seeding::Seeding;
    use crate::matches_to_play::MatchesToPlay;
    use crate::next_opponent::NextOpponentInBracket;
    use crate::single_elimination_bracket::SingleEliminationBracket;
    use crate::{
        bracket::matches::single_elimination_format::Step,
        opponent::Opponent,
        player::{Participants, Player},
    };

    fn assert_players_play_each_other(
        player_1: usize,
        player_2: usize,
        player_ids: &[Player],
        s: &dyn NextOpponentInBracket,
    ) {
        let Some((Some(Opponent::Player(next_opponent)), match_id_1)) =
            s.next_opponent_in_bracket(player_ids[player_1].get_id())
        else {
            panic!("No next opponent")
        };
        assert_eq!(next_opponent, player_ids[player_2].get_id());

        let Some((Some(Opponent::Player(next_opponent)), match_id_2)) =
            s.next_opponent_in_bracket(player_ids[player_2].get_id())
        else {
            panic!("No next opponent")
        };
        assert_eq!(next_opponent, player_ids[player_1].get_id());

        assert_eq!(
            match_id_1, match_id_2,
            "expected player to be playing the same match"
        );
    }

    #[test]
    fn run_3_man() {
        todo!()
        // let mut p = vec![Player::new("don't use".into())]; // padding for readability
        // let mut seeding = vec![];
        // for i in 1..=3 {
        //     let player = Player::new(format!("p{i}"));
        //     p.push(player.clone());
        //     seeding.push(player.get_id());
        // }
        // let auto = true;
        // let seeding = Seeding::new(seeding).unwrap();
        // let seb = SingleEliminationBracket::create(seeding, auto);
        //
        // assert_eq!(seb.get_matches().len(), 2);
        // assert_eq!(seb.matches_to_play().len(), 1);
        // assert_players_play_each_other(2, 3, &p, &seb);
        // let (matches, _, new_matches) = seb
        //     .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
        //     .expect("bracket");
        // let s = Step::new(matches, &s.seeding, auto);
        // assert_eq!(new_matches.len(), 1, "grand finals match generated");
        // assert_players_play_each_other(1, 2, &p, &s);
        // assert_eq!(s.matches_to_play().len(), 1);
        // let (matches, _, new_matches) = s
        //     .tournament_organiser_reports_result(p[1].get_id(), (0, 2), p[2].get_id())
        //     .expect("bracket");
        // let s = Step::new(matches, &s.seeding, auto);
        // assert!(s.matches_to_play().is_empty());
        // assert!(new_matches.is_empty());
        // assert!(s.is_over());
    }

    #[test]
    fn run_5_man_bracket() {
        todo!()
        // let mut p = vec![Player::new("don't use".into())]; // padding for readability
        // let mut bad_seeding = Participants::default();
        // let mut seeding = vec![];
        // for i in 1..=5 {
        //     let player = Player::new(format!("p{i}"));
        //     p.push(player.clone());
        //     seeding.push(player.get_id());
        //     bad_seeding = bad_seeding.add_participant(player).expect("bracket");
        // }
        // let auto = true;
        // let s = Step::create(&seeding, auto).expect("step");
        //
        // assert_eq!(s.matches.len(), 4);
        // assert_eq!(s.matches_to_play().len(), 2);
        // let (matches, _, _new_matches) = s
        //     .tournament_organiser_reports_result(p[4].get_id(), (2, 0), p[5].get_id())
        //     .expect("bracket");
        // let s = Step::new(matches, &s.seeding, auto);
        // assert_eq!(s.matches_to_play().len(), 2);
        // let (matches, _, _new_matches) = s
        //     .tournament_organiser_reports_result(p[2].get_id(), (0, 2), p[3].get_id())
        //     .expect("bracket");
        // let s = Step::new(matches, &s.seeding, auto);
        // assert_eq!(s.matches_to_play().len(), 1);
        // let (matches, _, _new_matches) = s
        //     .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[4].get_id())
        //     .expect("bracket");
        // let s = Step::new(matches, &s.seeding, auto);
        // assert_eq!(s.matches_to_play().len(), 1);
        // let (matches, _, _new_matches) = s
        //     .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[3].get_id())
        //     .expect("bracket");
        // let s = Step::new(matches, &s.seeding, auto);
        // assert!(s.is_over());
        // assert_eq!(s.matches_to_play().len(), 0);
    }
}
