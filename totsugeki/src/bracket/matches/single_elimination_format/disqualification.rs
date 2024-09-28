//! single elimination disqualification implementation

#[cfg(test)]
mod tests {

    use crate::{
        bracket::matches::{assert_outcome, single_elimination_format::Step, Error, Progression},
        opponent::Opponent,
        player::{Id as PlayerId, Participants, Player},
    };

    #[test]
    fn disqualifying_unknown_player_returns_error() {
        let mut p = vec![Player::new("don't use".into())];
        let mut bad_seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            bad_seeding = bad_seeding.add_participant(player).expect("seeding");
        }
        let seeding = bad_seeding
            .get_players_list()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<_>>();
        let auto = false;
        let s = Step::create(&seeding, auto).expect("step");

        let unknown_player = PlayerId::new_v4();
        let id = match s.disqualify_participant(unknown_player) {
            Err(Error::UnknownPlayer(id, _participants)) => id,
            Err(e) => panic!("Expected UnknownPlayer error, got {e:?}"),
            Ok((matches, _)) => panic!("Expected error, bracket: {matches:?}"),
        };
        assert_eq!(id, unknown_player);
    }

    #[test]
    fn disqualifying_player_that_could_not_make_it() {
        let mut p = vec![Player::new("don't use".into())];
        let mut bad_seeding = Participants::default();
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
            bad_seeding = bad_seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::create(&seeding, auto).expect("step");

        assert!(
            !s.matches.iter().any(
                |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id())
            ),
            "expected player 1 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[1].get_id())
            .expect("bracket with player 1 disqualified");
        let s = Step::new(matches, &s.seeding, auto);
        assert!(
            s.matches.iter().any(
                |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id())
            ),
            "expected match where player 1 is declared looser"
        );
        assert!(
            s.matches
                .iter()
                .any(|m| m.contains(p[2].get_id()) && m.contains(p[3].get_id())),
            "expected player 2 and 3 playing"
        );
    }

    #[test]
    fn disqualifying_player_sets_looser_of_their_current_match() {
        let mut p = vec![Player::new("don't use".into())];
        let mut bad_seeding = Participants::default();
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
            bad_seeding = bad_seeding.add_participant(player).expect("seeding");
        }
        let auto = false;
        let s = Step::create(&seeding, auto).expect("step");

        let (matches, match_id_p2, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("reported result by player 2");
        let s = Step::new(matches, &s.seeding, auto);
        let (matches, _) = s
            .validate_match_result(match_id_p2)
            .expect("validated match for p2 and p3");
        let s = Step::new(matches, &s.seeding, auto);

        assert!(
            !s.matches.iter().any(|m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())),
            "expected player 2 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[2].get_id())
            .expect("p2 is disqualified");
        let s = Step::new(matches, &s.seeding, auto);
        assert!(
            s.matches.iter().any(|m| matches!(
                    (m.get_automatic_loser(), m.get_winner()),
                    (Opponent::Player(loser), Opponent::Player(winner)) 
                    if loser == p[2].get_id() && winner == p[1].get_id())),
            "expected player 1 winning match where player 2 is disqualified, got {:?}",
            s.matches
        );
        assert!(
            s.matches
                .iter()
                .all(|m| m.get_winner() != Opponent::Unknown),
            "expected all matches were played"
        );
    }

    #[test]
    fn disqualifying_player_sets_their_opponent_as_the_winner_and_they_move_to_their_next_match() {
        let mut p = vec![Player::new("don't use".into())];
        let mut bad_seeding = Participants::default();
        let mut seeding = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding.push(player.get_id());
            bad_seeding = bad_seeding.add_participant(player).expect("seeding");
        }
        let auto = false;
        let s = Step::create(&seeding, auto).expect("step");

        assert!(
            !s.matches.iter().any(|m|
                matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
            ),
            "expected player 2 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[2].get_id())
            .expect("bracket with player 2 disqualified");
        let s = Step::new(matches, &s.seeding, auto);
        assert!(
            s.matches.iter().any(|m|
                matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
            ),
            "expected match where player 2 is declared looser"
        );
        assert!(
            s.matches
                .iter()
                .any(|m| m.contains(p[1].get_id()) && m.contains(p[3].get_id())),
            "expected player 1 and 3 playing in grand finals"
        );
    }
}
