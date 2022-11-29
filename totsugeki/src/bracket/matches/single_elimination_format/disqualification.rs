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
        let mut seeding = Participants::default();
        for i in 1..=3 {
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

        let unknown_player = PlayerId::new_v4();
        match s.disqualify_participant(unknown_player) {
            Ok((matches, _)) => panic!("Expected error, bracket: {matches:?}"),
            Err(e) => match e {
                Error::UnknownPlayer(id, _participants) => {
                    assert_eq!(id, unknown_player);
                }
                _ => panic!("Expected UnknownPlayer error, got {e:?}"),
            },
        }
    }

    #[test]
    fn disqualifying_player_that_could_not_make_it() {
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

        assert!(
            !s.matches.iter().any(|m| if m.contains(p[1].get_id()) {
                if let Opponent::Player(player) = m.get_automatic_loser() {
                    return player.get_id() == p[1].get_id();
                }
                false
            } else {
                false
            }),
            "expected player 1 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[1].get_id())
            .expect("bracket with player 1 disqualified");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert!(
            s.matches.iter().any(|m| if m.contains(p[1].get_id()) {
                if let Opponent::Player(player) = m.get_automatic_loser() {
                    return player.get_id() == p[1].get_id();
                }
                false
            } else {
                false
            }),
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
        let mut seeding = Participants::default();
        for i in 1..=3 {
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

        let (matches, match_id_p2, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("reported result by player 2");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        let (matches, _) = s
            .validate_match_result(match_id_p2)
            .expect("validated match for p2 and p3");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");

        assert!(
            !s.matches.iter().any(|m| if m.contains(p[2].get_id()) {
                if let Opponent::Player(player) = m.get_automatic_loser() {
                    return player.get_id() == p[2].get_id();
                }
                false
            } else {
                false
            }),
            "expected player 2 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[2].get_id())
            .expect("p2 is disqualified");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert!(
            s.matches.iter().any(|m| if m.contains(p[2].get_id()) {
                if let Opponent::Player(loser) = m.get_automatic_loser() {
                    if loser.get_id() == p[2].get_id() {
                        if let Opponent::Player(winner) = m.get_winner() {
                            return winner.get_id() == p[1].get_id();
                        }
                    }
                }
                false
            } else {
                false
            }),
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
        let mut seeding = Participants::default();
        for i in 1..=3 {
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

        assert!(
            !s.matches.iter().any(|m| if m.contains(p[2].get_id()) {
                if let Opponent::Player(player) = m.get_automatic_loser() {
                    return player.get_id() == p[2].get_id();
                }
                false
            } else {
                false
            }),
            "expected player 2 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[2].get_id())
            .expect("bracket with player 2 disqualified");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert!(
            s.matches.iter().any(|m| if m.contains(p[2].get_id()) {
                if let Opponent::Player(player) = m.get_automatic_loser() {
                    return player.get_id() == p[2].get_id();
                }
                false
            } else {
                false
            }),
            "expected match where player 2 is declared looser"
        );
        assert!(
            s.matches
                .iter()
                .any(|m| m.contains(p[1].get_id()) && m.contains(p[3].get_id())),
            "expected player 1 and 3 playing in grand finals"
        );
    }

    #[test]
    fn disqualifying_everyone_is_impossible_because_the_last_player_remaining_wins_grand_finals_automatically(
    ) {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
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
        let (matches, _) = s
            .disqualify_participant(p[2].get_id())
            .expect("bracket with player 2 disqualified");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[7], &p[2]);
        let (matches, _) = s
            .disqualify_participant(p[3].get_id())
            .expect("bracket with player 3 disqualified");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[6], &p[3]);
        let (matches, _) = s
            .disqualify_participant(p[4].get_id())
            .expect("bracket with player 4 disqualified");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[5], &p[4]);
        let (matches, _) = s
            .disqualify_participant(p[5].get_id())
            .expect("bracket with player 5 disqualified");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        // player 5 opponent is unknown
        let (matches, _) = s
            .disqualify_participant(p[6].get_id())
            .expect("bracket with player 6 disqualified");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[7], &p[6]);
        let (matches, _) = s
            .disqualify_participant(p[7].get_id())
            .expect("bracket with player 7 disqualified");
        let s = Step::new(Some(matches), seeding.clone(), s.seeding, auto).expect("step");
        // player 7 is in GF
        let (matches, _) = s
            .disqualify_participant(p[8].get_id())
            .expect("bracket with player 8 disqualified");
        let s = Step::new(Some(matches), seeding, s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[1], &p[8]);
        assert_outcome(&s.matches, &p[1], &p[5]);
        assert_outcome(&s.matches, &p[1], &p[7]);

        match s.disqualify_participant(p[1].get_id()) {
            Ok(_) => panic!("Expected error but none returned: {s:?}"),
            Err(e) => match e {
                Error::TournamentIsOver => {}
                _ => panic!("Expected Tournament over error but got {e:?}"),
            },
        };
    }
}
