//! double elimination bracket disqualification tests

#[cfg(test)]
mod tests {
    use crate::{
        bracket::matches::{
            assert_outcome, assert_x_wins_against_y, double_elimination_format::Step, Error,
            Progression,
        },
        matches::partition_double_elimination_matches,
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
        let s = Step::new(None, seeding, auto).expect("step");

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
        let s = Step::new(None, seeding.clone(), auto).expect("step");

        assert!(
            !s.bracket.iter().any(|m| if m.contains(p[1].get_id()) {
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
        let s = Step::new(Some(matches), seeding, auto).expect("step");
        assert!(
            s.bracket.iter().any(|m| if m.contains(p[1].get_id()) {
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
            s.bracket
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
        let s = Step::new(None, seeding.clone(), auto).expect("step");

        let (matches, match_id_p2, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("reported result by player 2");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        let (matches, _) = s
            .validate_match_result(match_id_p2)
            .expect("validated match for p2 and p3");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");

        assert!(
            !s.bracket.iter().any(|m| if m.contains(p[2].get_id()) {
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
        let s = Step::new(Some(matches), seeding, auto).expect("step");

        let condition = s.bracket.iter().any(|m| {
            if m.contains(p[2].get_id()) {
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
            }
        });
        assert!(
            condition,
            "expected player 1 winning match where player 2 is disqualified, got {}",
            s.bracket
                .iter()
                .find(|m| m.contains(p[1].get_id()) && m.contains(p[2].get_id()))
                .expect("m")
                .get_debug_summary()
        );
        assert!(
            !s.is_over(),
            "as opposed to single elimination, bracket is not over"
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
        let s = Step::new(None, seeding.clone(), auto).expect("step");

        assert!(
            !s.bracket.iter().any(|m| if m.contains(p[2].get_id()) {
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
        let s = Step::new(Some(matches), seeding, auto).expect("step");
        assert!(
            s.bracket.iter().any(|m| if m.contains(p[2].get_id()) {
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
            s.bracket
                .iter()
                .any(|m| m.contains(p[1].get_id()) && m.contains(p[3].get_id())),
            "expected player 1 and 3 playing in winner finals"
        );
    }

    fn assert_player_drops_to_losers(s: &Step, n: usize, p: &[Player]) {
        let (winners, losers, _, _) =
            partition_double_elimination_matches(&s.bracket, s.seeding.len()).expect("partition");
        assert!(
            !winners.iter().any(|m| m.contains(p[n].get_id())
                && m.get_winner() == Opponent::Unknown
                && m.get_automatic_loser() == Opponent::Unknown),
            "expected player {n} having no matches in winners"
        );
        assert!(
            losers.iter().any(|m| m.contains(p[n].get_id())),
            "expected player {n} in losers",
        );
    }

    fn initial_step(n: usize, auto: bool) -> (Step, Participants, Vec<Player>) {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=n {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        (
            Step::new(None, seeding.clone(), auto).expect("step"),
            seeding,
            p,
        )
    }

    #[test]
    fn disqualifying_everyone_is_impossible_because_the_last_player_remaining_wins_grand_finals_automatically(
    ) {
        let auto = true;
        let (s, seeding, p) = initial_step(8, auto);

        let (matches, _) = s.disqualify_participant(p[2].get_id()).expect("p2 DQ'ed");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        assert_player_drops_to_losers(&s, 2, &p);
        assert_outcome(&s.bracket, &p[7], &p[2]);

        let (matches, _) = s.disqualify_participant(p[3].get_id()).expect("p3 DQ'ed");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        assert_player_drops_to_losers(&s, 3, &p);
        assert_outcome(&s.bracket, &p[6], &p[3]);

        let (matches, _) = s.disqualify_participant(p[4].get_id()).expect("p4 DQ'ed");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        assert_outcome(&s.bracket, &p[5], &p[4]);
        assert_player_drops_to_losers(&s, 4, &p);
        let (_, l_bracket, _, _) =
            partition_double_elimination_matches(&s.bracket, s.seeding.len()).expect("partition");
        assert_eq!(
            l_bracket
                .iter()
                .filter(|m| m.contains(p[4].get_id()))
                .count(),
            1
        );

        let (matches, _) = s.disqualify_participant(p[5].get_id()).expect("p5 DQ'ed");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        // player 5 opponent in winners is unknown yet he can drop to losers
        // already, even if 1vs8 has not been played out
        assert_player_drops_to_losers(&s, 5, &p);

        let (matches, _) = s.disqualify_participant(p[6].get_id()).expect("p6 DQ'ed");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        assert_outcome(&s.bracket, &p[7], &p[6]);

        let (matches, _) = s.disqualify_participant(p[7].get_id()).expect("p7 DQ'ed");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        assert_player_drops_to_losers(&s, 7, &p);
        assert!(&s
            .bracket
            .iter()
            .any(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3]));
        let (_w_bracket, l_bracket, _, _) =
            partition_double_elimination_matches(&s.bracket, s.seeding.len()).expect("p");
        let m = &l_bracket
            .iter()
            .find(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3])
            .expect("m");
        let Opponent::Player(loser) = m.get_automatic_loser() else {
            panic!("expected loser but found none {m:?}");
        };
        assert_eq!(loser.get_id(), p[7].get_id());

        let (matches, _) = s.disqualify_participant(p[8].get_id()).expect("p8 DQ'ed");
        let s = Step::new(Some(matches), seeding, auto).expect("step");
        assert_outcome(&s.bracket, &p[1], &p[8]);
        assert_player_drops_to_losers(&s, 8, &p);
        assert_outcome(&s.bracket, &p[8], &p[5]);
        assert_outcome(&s.bracket, &p[1], &p[5]);
        assert_player_drops_to_losers(&s, 5, &p);
        assert_outcome(&s.bracket, &p[1], &p[7]);
        assert_player_drops_to_losers(&s, 7, &p);
        // player 7 is in GF
        (2..=8).for_each(|i| {
            assert!(s.is_disqualified(p[i].get_id()), "player {i} disqualified");
        });
        assert!(
            !s.is_disqualified(p[1].get_id()),
            "player 1 not disqualified"
        );
        let (winner_bracket, loser_bracket, gf, _gf_reset) =
            partition_double_elimination_matches(&s.bracket, s.seeding.len()).expect("partition");
        for m in &winner_bracket {
            assert!(
                m.get_automatic_loser() != Opponent::Unknown,
                "expected winner bracket match to have automatic loser but got none: {m:?}"
            );
        }
        for m in &loser_bracket {
            assert_ne!(
                m.get_automatic_loser(),
                Opponent::Unknown,
                "expected loser bracket match to have automatic loser but got none: {m:?}"
            );
        }

        assert_outcome(&winner_bracket, &p[1], &p[8]);
        assert_outcome(
            &[loser_bracket.last().expect("loser bracket finals").clone()],
            &p[8],
            &p[7],
        );
        assert_outcome(&[gf], &p[1], &p[8]);

        // https://stackoverflow.com/a/68919527
        assert!(matches!(
            s.disqualify_participant(p[1].get_id()),
            Err(Error::TournamentIsOver)
        ));
    }

    #[test]
    fn disqualifying_most_in_double_elimination_tournament_and_lowest_expected_seed_in_winners_final(
    ) {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, seeding.clone(), auto).expect("step");

        let (matches, _) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        let (matches, _) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        let (_w_bracket, l_bracket, _, _) =
            partition_double_elimination_matches(&s.bracket, seeding.len()).expect("partition");
        assert!(
            l_bracket.iter().any(|m| {
                if let Opponent::Player(auto) = m.get_automatic_loser() {
                    auto.get_id() == p[7].id
                } else {
                    false
                }
            }),
            "p7 disqualified in losers"
        );
        let (matches, _) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        let (_, l_bracket, _, _) =
            partition_double_elimination_matches(&s.bracket, seeding.len()).expect("partition");
        assert_x_wins_against_y(&p[6], &p[7], &l_bracket);

        let (matches, _) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        let (matches, _) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        let (matches, _) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        let s = Step::new(Some(matches), seeding.clone(), auto).expect("step");
        let (matches, new_matches) = s.disqualify_participant(p[2].get_id()).expect("dq 2");
        let s = Step::new(Some(matches), seeding, auto).expect("step");

        assert_eq!(new_matches.len(), 0);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[1].get_id()),);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[2].get_id()),);
        assert_eq!(
            s.bracket[s.bracket.len() - 1].get_players(),
            [Opponent::Unknown, Opponent::Unknown],
            "expected no p in reset but got {:?}",
            s.bracket[s.bracket.len() - 1].get_players()
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_automatic_loser(),
            Opponent::Player(p[2].clone()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_winner(),
            Opponent::Player(p[1].clone()),
            "expected winner of grand finals to be {}\n{:?}",
            p[1],
            s.bracket[s.bracket.len() - 2],
        );
        assert!(s.is_over(), "expected s to be over but got {s:?}");
    }

    #[test]
    fn disqualify_from_winner() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, seeding.clone(), auto).expect("step");

        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq");
        let s = Step::new(Some(bracket), seeding, auto).expect("step");
        let new_matches = s.matches_to_play();
        assert_eq!(
            new_matches.len(),
            1,
            "expected 1 match after DQ'ing p3 in 3 player tournament"
        );
        let (_bracket, _, new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("to report");
        assert_eq!(new_matches.len(), 1, "expected 1 new match");

        assert!(
            new_matches[0].contains(p[1].get_id()),
            "expected player 1 in GF"
        );
        assert!(
            new_matches[0].contains(p[2].get_id()),
            "expected player 2 in GF"
        );
    }

    #[test]
    fn disqualify_in_double_elimination_bracket_from_loser() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, seeding.clone(), auto).expect("step");

        let (bracket, _, new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("to report");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        assert_eq!(new_matches.len(), 1, "expected 1 new match");
        let (bracket, _, new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("to report");
        assert_eq!(new_matches.len(), 1, "expected 1 new match");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[3].get_id()).expect("dq");
        assert_eq!(new_matches.len(), 1);
        assert!(
            new_matches[0].contains(p[1].get_id()),
            "expected player 1 in GF"
        );
        assert!(
            new_matches[0].contains(p[2].get_id()),
            "expected player 2 in GF"
        );
        let s = Step::new(Some(bracket), seeding, auto).expect("step");

        let new_matches = s.matches_to_play();
        assert_eq!(new_matches.len(), 1);

        assert!(
            new_matches[0].contains(p[1].get_id()),
            "expected player 1 in GF"
        );
        assert!(
            new_matches[0].contains(p[2].get_id()),
            "expected player 2 in GF"
        );
    }

    #[test]
    fn disqualifying_everyone_in_double_elimination_tournament_is_imposible() {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, seeding.clone(), auto).expect("step");
        let (bracket, new_matches) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        assert_eq!(new_matches.len(), 1);
        assert!(new_matches[0].contains(p[2].get_id()), "expected {}", p[2]);
        assert!(new_matches[0].contains(p[3].get_id()), "expected {}", p[3]);
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        assert_eq!(new_matches.len(), 1);
        assert!(
            new_matches[0].contains(p[1].get_id()),
            "expected {} in new match after disqualifying {}",
            p[1],
            p[6]
        );
        assert!(
            new_matches[0].contains(p[4].get_id()),
            "expected {} in new match after disqualifying {}",
            p[4],
            p[6]
        );
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        assert_eq!(new_matches.len(), 1);
        assert!(
            new_matches[0].contains(p[1].get_id()),
            "expected {} in winner finals after disqualifying {}",
            p[1],
            p[3]
        );
        assert!(
            new_matches[0].contains(p[2].get_id()),
            "expected {} in winner finals after disqualifying {}",
            p[2],
            p[6]
        );
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[2].get_id()).expect("dq 2");
        let s = Step::new(Some(bracket), seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[1].get_id()),);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[2].get_id()),);
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_automatic_loser(),
            Opponent::Player(p[2].clone()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_winner(),
            Opponent::Player(p[1].clone()),
            "expected winner of grand finals to be {}\n{:?}",
            p[1],
            s.bracket[s.bracket.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.bracket
        );
    }

    #[test]
    fn disqualifying_most_in_double_elimination_tournament_and_grand_finalist_from_winner_in_grand_finals(
    ) {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("player 1 wins in winners finals");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[1].get_id()).expect("dq 1");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), seeding, auto).expect("step");
        assert!(s.bracket[s.bracket.len() - 2].contains(p[1].get_id()),);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[2].get_id()),);
        assert!(
            s.bracket[s.bracket.len() - 1].contains(p[1].get_id()),
            "expected player 1 in reset",
        );
        assert!(s.bracket[s.bracket.len() - 1].contains(p[2].get_id()),);
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_automatic_loser(),
            Opponent::Player(p[1].clone()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_winner(),
            Opponent::Player(p[2].clone()),
            "expected winner of grand finals to be {}\n{:?}",
            p[1],
            s.bracket[s.bracket.len() - 2],
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 1].get_automatic_loser(),
            Opponent::Player(p[1].clone()),
            "expected automatic loser of reset to be {}",
            p[1]
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 1].get_winner(),
            Opponent::Player(p[2].clone()),
            "expected winner of reset to be {}\n{:?}",
            p[2],
            s.bracket[s.bracket.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.bracket
        );
    }

    #[test]
    fn disqualifying_most_in_double_elimination_tournament_and_grand_finalist_from_loser_in_grand_finals(
    ) {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("player 1 wins in winners finals");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[2].get_id()).expect("dq 2");
        let s = Step::new(Some(bracket), seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[1].get_id()),);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[2].get_id()),);
        assert_eq!(
            s.bracket[s.bracket.len() - 1].get_players(),
            [Opponent::Unknown, Opponent::Unknown]
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_automatic_loser(),
            Opponent::Player(p[2].clone()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_winner(),
            Opponent::Player(p[1].clone()),
            "expected winner of grand finals to be {}\n{:?}",
            p[1],
            s.bracket[s.bracket.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.bracket
        );
    }

    #[test]
    fn disqualifying_most_in_double_elimination_tournament_and_highest_expected_seed_in_winners_final(
    ) {
        let mut p = vec![Player::new("don't use".into())];
        let mut seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            seeding = seeding.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        let s = Step::new(Some(bracket), seeding.clone(), auto).expect("step");
        let (bracket, new_matches) = s.disqualify_participant(p[1].get_id()).expect("dq 1");
        let s = Step::new(Some(bracket), seeding, auto).expect("step");

        assert_eq!(new_matches.len(), 0);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[1].get_id()),);
        assert!(s.bracket[s.bracket.len() - 2].contains(p[2].get_id()),);
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_players(),
            [
                Opponent::Player(p[2].clone()),
                Opponent::Player(p[1].clone())
            ],
            "expected player 1 and 2 in grand finals but got {:?}",
            s.bracket[s.bracket.len() - 2].get_players()
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_automatic_loser(),
            Opponent::Player(p[1].clone()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.bracket[s.bracket.len() - 2].get_winner(),
            Opponent::Player(p[2].clone()),
            "expected winner of reset to be {}\n{:?}",
            p[2],
            s.bracket[s.bracket.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.bracket
        );
    }
}
