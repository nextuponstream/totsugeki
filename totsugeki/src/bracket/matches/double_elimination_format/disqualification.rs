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
        let mut participants = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            participants = participants.add_participant(player).expect("seeding");
        }
        let auto = false;
        let bracket = Step::new(None, participants.get_seeding(), auto).expect("step");

        let unknown_player = PlayerId::new_v4();
        let id = match bracket.disqualify_participant(unknown_player) {
            Err(Error::UnknownPlayer(id, _participants)) => id,
            Err(e) => panic!("Expected UnknownPlayer error, got {e:?}"),
            Ok((matches, _)) => panic!("Expected error but got none, bracket: {matches:?}"),
        };
        assert_eq!(id, unknown_player);
    }

    #[test]
    fn disqualifying_player_that_could_not_make_it() {
        let mut p = vec![Player::new("don't use".into())];
        let mut participants = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            participants = participants.add_participant(player).expect("seeding");
        }
        let auto = true;
        let s = Step::new(None, participants.get_seeding(), auto).expect("step");

        assert!(
            !s.matches.iter().any(
                |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id()) 
            ),
            "expected player 1 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[1].get_id())
            .expect("bracket with player 1 disqualified");
        let bracket = Step::new(Some(matches), participants.get_seeding(), auto).expect("step");
        assert!(
            bracket.matches.iter().any(
                |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[1].get_id()) 
            ),
            "expected match where player 1 is declared looser"
        );
        assert!(
            bracket
                .matches
                .iter()
                .any(|m| m.contains(p[2].get_id()) && m.contains(p[3].get_id())),
            "expected player 2 and 3 playing"
        );
    }

    #[test]
    fn disqualifying_player_sets_looser_of_their_current_match() {
        let mut p = vec![Player::new("don't use".into())];
        let mut participants = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            participants = participants.add_participant(player).expect("seeding");
        }
        let auto = false;
        let s = Step::new(None, participants.get_seeding(), auto).expect("step");

        let (matches, match_id_p2, _new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("reported result by player 2");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");
        let (matches, _) = s
            .validate_match_result(match_id_p2)
            .expect("validated match for p2 and p3");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");

        assert!(
            !s.matches.iter().any(|m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())),
            "expected player 2 not to be declared looser in any match"
        );
        let (matches, _) = s
            .disqualify_participant(p[2].get_id())
            .expect("p2 is disqualified");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");

        let condition = s
            .matches
            .iter()
            .any(|m| match (m.get_automatic_loser(), m.get_winner()) {
                (Opponent::Player(loser), Opponent::Player(winner)) if loser == p[2].get_id() => {
                    winner == p[1].get_id()
                }
                _ => false,
            });
        assert!(
            condition,
            "expected player 1 winning match where player 2 is disqualified, got {}",
            s.matches
                .iter()
                .find(|m| m.contains(p[1].get_id()) && m.contains(p[2].get_id()))
                .expect("m")
                .summary()
        );
        assert!(
            !s.is_over(),
            "as opposed to single elimination, bracket is not over"
        );
    }

    #[test]
    fn disqualifying_player_sets_their_opponent_as_the_winner_and_they_move_to_their_next_match() {
        let mut p = vec![Player::new("don't use".into())];
        let mut participants = Participants::default();
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            participants = participants.add_participant(player).expect("seeding");
        }
        let auto = false;
        let bracket = Step::new(None, participants.get_seeding(), auto).expect("step");

        assert!(
            !bracket.matches.iter().any(
                |m| matches!(m.get_automatic_loser(), Opponent::Player(loser) if loser == p[2].get_id())
            ),
            "expected player 2 not to be declared looser in any match"
        );
        let (matches, _) = bracket
            .disqualify_participant(p[2].get_id())
            .expect("bracket with player 2 disqualified");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        assert!(
            bracket
                .matches
                .iter()
                .any(|m| matches!(m.get_automatic_loser(), Opponent::Player(player) if player == p[2].get_id())),
            "expected match where player 2 is declared looser"
        );
        assert!(
            bracket
                .matches
                .iter()
                .any(|m| m.contains(p[1].get_id()) && m.contains(p[3].get_id())),
            "expected player 1 and 3 playing in winner finals"
        );
    }

    fn assert_player_drops_to_losers(s: &Step, n: usize, p: &[Player]) {
        let (winners, losers, _, _) =
            partition_double_elimination_matches(&s.matches, s.seeding.len());
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
        let mut participants = Participants::default();
        for i in 1..=n {
            let player = Player::new(format!("p{i}"));
            p.push(player.clone());
            participants = participants.add_participant(player).expect("seeding");
        }
        (
            Step::new(None, participants.get_seeding(), auto).expect("step"),
            participants,
            p,
        )
    }

    #[test]
    fn disqualifying_everyone_is_impossible_because_the_last_player_remaining_wins_grand_finals_automatically(
    ) {
        let auto = true;
        let (bracket, _seeding, p) = initial_step(8, auto);

        let (matches, _) = bracket
            .disqualify_participant(p[2].get_id())
            .expect("p2 DQ'ed");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        assert_player_drops_to_losers(&bracket, 2, &p);
        assert_outcome(&bracket.matches, &p[7], &p[2]);

        let (matches, _) = bracket
            .disqualify_participant(p[3].get_id())
            .expect("p3 DQ'ed");
        let s = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        assert_player_drops_to_losers(&s, 3, &p);
        assert_outcome(&s.matches, &p[6], &p[3]);

        let (matches, _) = s.disqualify_participant(p[4].get_id()).expect("p4 DQ'ed");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[5], &p[4]);
        assert_player_drops_to_losers(&s, 4, &p);
        let (_, l_bracket, _, _) =
            partition_double_elimination_matches(&s.matches, s.seeding.len());
        assert_eq!(
            l_bracket
                .iter()
                .filter(|m| m.contains(p[4].get_id()))
                .count(),
            1
        );

        let (matches, _) = s.disqualify_participant(p[5].get_id()).expect("p5 DQ'ed");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");
        // player 5 opponent in winners is unknown yet he can drop to losers
        // already, even if 1vs8 has not been played out
        assert_player_drops_to_losers(&s, 5, &p);

        let (matches, _) = s.disqualify_participant(p[6].get_id()).expect("p6 DQ'ed");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[7], &p[6]);

        let (matches, _) = s.disqualify_participant(p[7].get_id()).expect("p7 DQ'ed");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");
        assert_player_drops_to_losers(&s, 7, &p);
        assert!(&s
            .matches
            .iter()
            .any(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3]));
        let (_w_bracket, l_bracket, _, _) =
            partition_double_elimination_matches(&s.matches, s.seeding.len());
        let m = &l_bracket
            .iter()
            .find(|m| m.contains(p[7].get_id()) && m.get_seeds() == [2, 3])
            .expect("m");
        let Opponent::Player(loser) = m.get_automatic_loser() else {
            panic!("expected loser but found none {m:?}");
        };
        assert_eq!(loser, p[7].get_id());

        let (matches, _) = s.disqualify_participant(p[8].get_id()).expect("p8 DQ'ed");
        let s = Step::new(Some(matches), s.seeding, auto).expect("step");
        assert_outcome(&s.matches, &p[1], &p[8]);
        assert_player_drops_to_losers(&s, 8, &p);
        assert_outcome(&s.matches, &p[8], &p[5]);
        assert_outcome(&s.matches, &p[1], &p[5]);
        assert_player_drops_to_losers(&s, 5, &p);
        assert_outcome(&s.matches, &p[1], &p[7]);
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
            partition_double_elimination_matches(&s.matches, s.seeding.len());
        for m in &winner_bracket {
            assert_ne!(
                m.get_automatic_loser(),
                Opponent::Unknown,
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
            &[*loser_bracket.last().expect("loser bracket finals")],
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
        let mut players = vec![Player::new("don't use".into())];
        let mut p = vec![PlayerId::new_v4()];
        let mut bad_seeding = Participants::default();
        for i in 1..=8 {
            let player = Player::new(format!("p{i}"));
            players.push(player.clone());
            p.push(player.get_id());
            bad_seeding = bad_seeding.add_participant(player).expect("seeding");
        }
        let seeding = bad_seeding
            .get_players_list()
            .iter()
            .map(Player::get_id)
            .collect::<Vec<_>>();
        let auto = true;
        let bracket = Step::new(None, seeding, auto).expect("step");

        let (matches, _) = bracket.disqualify_participant(p[8]).expect("dq 8");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        let (matches, _) = bracket
            .disqualify_participant(players[7].get_id())
            .expect("dq 7");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        let (_w_bracket, l_bracket, _, _) =
            partition_double_elimination_matches(&bracket.matches, bad_seeding.len());
        assert!(
            l_bracket.iter().any(|m| {
                let Opponent::Player(auto) = m.get_automatic_loser() else {
                    return false;
                };
                auto == p[7]
            }),
            "p7 disqualified in losers"
        );
        let (matches, _) = bracket
            .disqualify_participant(players[6].get_id())
            .expect("dq 6");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        let (_, l_bracket, _, _) =
            partition_double_elimination_matches(&bracket.matches, bad_seeding.len());
        assert_x_wins_against_y(&players[6], &players[7], &l_bracket);

        let (matches, _) = bracket
            .disqualify_participant(players[5].get_id())
            .expect("dq 5");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        let (matches, _) = bracket
            .disqualify_participant(players[4].get_id())
            .expect("dq 4");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        let (matches, _) = bracket
            .disqualify_participant(players[3].get_id())
            .expect("dq 3");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");
        let (matches, new_matches) = bracket
            .disqualify_participant(players[2].get_id())
            .expect("dq 2");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("step");

        assert_eq!(new_matches.len(), 0);
        assert!(bracket.matches[bracket.matches.len() - 2].contains(players[1].get_id()),);
        assert!(bracket.matches[bracket.matches.len() - 2].contains(players[2].get_id()),);
        assert_eq!(
            bracket.matches[bracket.matches.len() - 1].get_players(),
            [Opponent::Unknown, Opponent::Unknown],
            "expected no p in reset but got {:?}",
            bracket.matches[bracket.matches.len() - 1].get_players()
        );
        assert_eq!(
            bracket.matches[bracket.matches.len() - 2].get_automatic_loser(),
            Opponent::Player(players[2].get_id()),
            "expected automatic loser of grand finals to be {}",
            players[2]
        );
        assert_eq!(
            bracket.matches[bracket.matches.len() - 2].get_winner(),
            Opponent::Player(players[1].get_id()),
            "expected winner of grand finals to be {}\n{:?}",
            players[1],
            bracket.matches[bracket.matches.len() - 2],
        );
        assert!(
            bracket.is_over(),
            "expected s to be over but got {bracket:?}"
        );
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
        let s = Step::new(
            None,
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("step");

        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
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
        let s = Step::new(
            None,
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("step");

        let (bracket, _, new_matches) = s
            .tournament_organiser_reports_result(p[2].get_id(), (2, 0), p[3].get_id())
            .expect("to report");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 1, "expected 1 new match");
        let (bracket, _, new_matches) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("to report");
        assert_eq!(new_matches.len(), 1, "expected 1 new match");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

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
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

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
        let s = Step::new(
            None,
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("step");
        let (bracket, new_matches) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        assert_eq!(new_matches.len(), 1);
        assert!(new_matches[0].contains(p[2].get_id()), "expected {}", p[2]);
        assert!(new_matches[0].contains(p[3].get_id()), "expected {}", p[3]);
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

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
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

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
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[2].get_id()).expect("dq 2");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert!(s.matches[s.matches.len() - 2].contains(p[1].get_id()),);
        assert!(s.matches[s.matches.len() - 2].contains(p[2].get_id()),);
        assert_eq!(
            s.matches[s.matches.len() - 2].get_automatic_loser(),
            Opponent::Player(p[2].get_id()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.matches[s.matches.len() - 2].get_winner(),
            Opponent::Player(p[1].get_id()),
            "expected winner of grand finals to be {}\n{:?}",
            p[1],
            s.matches[s.matches.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.matches
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
        let s = Step::new(
            None,
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("step");
        let (bracket, _) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("player 1 wins in winners finals");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[1].get_id()).expect("dq 1");
        assert_eq!(new_matches.len(), 0);
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        assert!(s.matches[s.matches.len() - 2].contains(p[1].get_id()),);
        assert!(s.matches[s.matches.len() - 2].contains(p[2].get_id()),);
        assert!(
            s.matches[s.matches.len() - 1].contains(p[1].get_id()),
            "expected player 1 in reset",
        );
        assert!(s.matches[s.matches.len() - 1].contains(p[2].get_id()),);
        assert_eq!(
            s.matches[s.matches.len() - 2].get_automatic_loser(),
            Opponent::Player(p[1].get_id()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.matches[s.matches.len() - 2].get_winner(),
            Opponent::Player(p[2].get_id()),
            "expected winner of grand finals to be {}\n{:?}",
            p[1],
            s.matches[s.matches.len() - 2],
        );
        assert_eq!(
            s.matches[s.matches.len() - 1].get_automatic_loser(),
            Opponent::Player(p[1].get_id()),
            "expected automatic loser of reset to be {}",
            p[1]
        );
        assert_eq!(
            s.matches[s.matches.len() - 1].get_winner(),
            Opponent::Player(p[2].get_id()),
            "expected winner of reset to be {}\n{:?}",
            p[2],
            s.matches[s.matches.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.matches
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
        let s = Step::new(
            None,
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("step");
        let (bracket, _) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _, _) = s
            .tournament_organiser_reports_result(p[1].get_id(), (2, 0), p[2].get_id())
            .expect("player 1 wins in winners finals");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

        let (bracket, new_matches) = s.disqualify_participant(p[2].get_id()).expect("dq 2");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        assert_eq!(new_matches.len(), 0);
        assert!(s.matches[s.matches.len() - 2].contains(p[1].get_id()),);
        assert!(s.matches[s.matches.len() - 2].contains(p[2].get_id()),);
        assert_eq!(
            s.matches[s.matches.len() - 1].get_players(),
            [Opponent::Unknown, Opponent::Unknown]
        );
        assert_eq!(
            s.matches[s.matches.len() - 2].get_automatic_loser(),
            Opponent::Player(p[2].get_id()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.matches[s.matches.len() - 2].get_winner(),
            Opponent::Player(p[1].get_id()),
            "expected winner of grand finals to be {}\n{:?}",
            p[1],
            s.matches[s.matches.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.matches
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
        let s = Step::new(
            None,
            seeding
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            auto,
        )
        .expect("step");
        let (bracket, _) = s.disqualify_participant(p[8].get_id()).expect("dq 8");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[7].get_id()).expect("dq 7");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[6].get_id()).expect("dq 6");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[5].get_id()).expect("dq 5");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[4].get_id()).expect("dq 4");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, _) = s.disqualify_participant(p[3].get_id()).expect("dq 3");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");
        let (bracket, new_matches) = s.disqualify_participant(p[1].get_id()).expect("dq 1");
        let s = Step::new(Some(bracket), s.seeding, auto).expect("step");

        assert_eq!(new_matches.len(), 0);
        assert!(s.matches[s.matches.len() - 2].contains(p[1].get_id()),);
        assert!(s.matches[s.matches.len() - 2].contains(p[2].get_id()),);
        assert_eq!(
            s.matches[s.matches.len() - 2].get_players(),
            [
                Opponent::Player(p[2].get_id()),
                Opponent::Player(p[1].get_id())
            ],
            "expected player 1 and 2 in grand finals but got {:?}",
            s.matches[s.matches.len() - 2].get_players()
        );
        assert_eq!(
            s.matches[s.matches.len() - 2].get_automatic_loser(),
            Opponent::Player(p[1].get_id()),
            "expected automatic loser of grand finals to be {}",
            p[2]
        );
        assert_eq!(
            s.matches[s.matches.len() - 2].get_winner(),
            Opponent::Player(p[2].get_id()),
            "expected winner of reset to be {}\n{:?}",
            p[2],
            s.matches[s.matches.len() - 2],
        );
        assert!(
            s.is_over(),
            "expected bracket to be over but got {:?}",
            s.matches
        );
    }
    #[test]
    fn fuzzer_incident_01() {
        // DQ'ed player is in GF

        //     Win(false)
        // [1, 4] af286c3a-2be7-44ca-83ba-3b739f806aec
        //     Win(false)
        // [2, 3] 9d66fdf4-3c66-466f-9683-8e21357400fe
        //     Disqualification(false)
        // [1, 2] c469cb9c-d1f3-4acd-854d-e5a00a76a0d1
        //     Disqualification(false)
        // [3, 4] 3717b34e-fc15-42d6-89f1-8aaa64210593
        //     Disqualification(false) SKIPPED
        // [2, 3] 32d326ea-5be8-4baa-a28a-b238810dcd6d
        // Disqualification(false)
        // [1, 2] 2556a287-3a98-44e6-8ad1-42ab66baa723
        // * [1, 4] -p1 VS Wp4
        // * [2, 3] -p2 VS Wp3
        // * [1, 2] Wp4 VS Lp3
        // * [3, 4] Wp2 VS Lp1
        // * [2, 3] Lp3 VS Wp2
        // * [1, 2] -p4 VS -p2
        // * [1, 2] -?  VS -?
        let mut p = vec![PlayerId::new_v4()];
        let mut seeding = vec![];
        for _ in 1..=4 {
            let id = PlayerId::new_v4();
            p.push(id);
            seeding.push(id);
        }
        let auto = true;
        let bracket = Step::new(None, seeding, auto).expect("bracket");
        let (matches, _, _) = bracket
            .tournament_organiser_reports_result(p[1], (0, 2), p[4])
            .expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _, _) = bracket
            .tournament_organiser_reports_result(p[2], (0, 2), p[3])
            .expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[3]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[1]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[2]).expect("bracket");
        let _bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
    }

    #[test]
    fn fuzzer_incident_02() {
        // DQ'ed player is in loser bracket already

        // #total players: 8
        // #events: 16
        // Win(true)
        // Disqualification(true)
        // TOWin(true)
        // Disqualification(false)
        // Disqualification(false)
        // Disqualification(false)
        // Disqualification(false)
        // Disqualification(false) // SKIPPED
        // Disqualification(false) // SKIPPED
        // Disqualification(false) // SKIPPED
        // Disqualification(false) // SKIPPED
        // Disqualification(false) // PANIC
        // before crash:
        // * [1, 8] Wp1 VS -p8
        // * [2, 7] Lp2 VS Wp7
        // * [3, 6] Wp3 VS -p6
        // * [4, 5] Wp4 VS Lp5
        // * [1, 4] Wp1 VS Lp4
        // * [2, 3] Wp7 VS Lp3
        // * [1, 2] Wp1 VS Lp7
        // * [5, 8] Lp5 VS Wp8
        // * [6, 7] Wp6 VS Lp2
        // * [3, 6] Lp3 VS Wp6
        // * [4, 5] Lp4 VS Wp8
        // * [3, 4] -p6 VS -p8
        // * [2, 3] Lp7 VS -?
        // * [1, 2] -p1 VS -?
        // * [1, 2] -?  VS -?
        let mut p = vec![PlayerId::new_v4()];
        let mut seeding = vec![];
        for _ in 1..=8 {
            let id = PlayerId::new_v4();
            p.push(id);
            seeding.push(id);
        }
        let auto = true;
        let bracket = Step::new(None, seeding, auto).expect("bracket");
        let (matches, _, _) = bracket
            .tournament_organiser_reports_result(p[1], (2, 0), p[8])
            .expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[2]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _, _) = bracket
            .tournament_organiser_reports_result(p[3], (2, 0), p[6])
            .expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[5]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[4]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[3]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[7]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[8]).expect("bracket");
        let _bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
    }

    #[test]
    fn fuzzer_incident_03() {
        //match 0:TOWin(true)
        // 	* p1
        // 	* p2
        // 	* p3
        // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
        // 	* [1, 2] -p1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
        // 	* [2, 3] -?  VS -p3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
        // 	* [1, 2] -?  VS -?  | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
        // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
        // match 1:TOWin(true)

        // 	* p1
        // 	* p2
        // 	* p3
        // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
        // 	* [1, 2] Wp1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
        // 	* [2, 3] -p2 VS -p3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
        // 	* [1, 2] -p1 VS -?  | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
        // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
        // match 3:Disqualification(true)

        // 	* p1
        // 	* p2
        // 	* p3
        // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
        // 	* [1, 2] Wp1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
        // 	* [2, 3] -p2 VS -p3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
        // 	* [1, 2] Lp1 VS -?  | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
        // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
        // match 2:Disqualification(false)

        // 	* p1
        // 	* p2
        // 	* p3
        // 	* [2, 3] Wp2 VS -p3 | match id: d7fab671-3d30-48f1-9b22-1494ede03de7
        // 	* [1, 2] Wp1 VS -p2 | match id: df25bc3f-f584-4fab-808c-6bf00c56755b
        // 	* [2, 3] Wp2 VS Lp3 | match id: ac034a6c-1a4d-4cba-9bd9-2acca62c5142
        // 	* [1, 2] Lp1 VS -p2 | match id: b725b8fd-b436-43d7-ba03-30ca9ac49c43
        // 	* [1, 2] -?  VS -?  | match id: a61b5bd7-dcad-44c4-bf83-48ebd34b9e30
        let mut p = vec![PlayerId::new_v4()];
        let mut seeding = vec![];
        for _ in 1..=3 {
            let id = PlayerId::new_v4();
            p.push(id);
            seeding.push(id);
        }
        let auto = true;
        let bracket = Step::new(None, seeding, auto).expect("bracket");
        let (matches, _, _) = bracket
            .tournament_organiser_reports_result(p[2], (2, 0), p[3])
            .expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _, _) = bracket
            .tournament_organiser_reports_result(p[1], (2, 0), p[2])
            .expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        let (matches, _) = bracket.disqualify_participant(p[1]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        // when moved into grand finals with someone disqualified, match should
        // be updated
        let (matches, _) = bracket.disqualify_participant(p[3]).expect("bracket");
        let bracket = Step::new(Some(matches), bracket.seeding, auto).expect("bracket");
        assert!(bracket.is_over());
    }
}
