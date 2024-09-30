//! single elimination disqualification implementation

#[cfg(test)]
mod tests {

    use crate::{
        bracket::matches::{assert_outcome, single_elimination_format::Step, Error, Progression},
        opponent::Opponent,
        player::{Id as PlayerId, Participants, Player},
    };

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
