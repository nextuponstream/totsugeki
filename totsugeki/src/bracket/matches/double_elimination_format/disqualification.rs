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
