//! Disqualification of player in bracket and all side-effects

#[cfg(test)]
mod tests {
    use crate::bracket::seeding::Seeding;
    use crate::double_elimination_bracket::DoubleEliminationBracket;
    use crate::matches::result::MatchFormat;
    use crate::player::Participants;
    use crate::player::Player;
    use crate::validation::AutomaticMatchValidationMode;
    use std::time::Instant;

    // cargo t disqualify_8000 -- --include-ignored --nocapture
    #[test]
    #[ignore]
    fn disqualify_8000_player_bracket() {
        let start = Instant::now();
        let _participants = Participants::default();
        let mut players = vec![];
        let mut seeding = vec![];

        for i in 1..=8000 {
            let p = Player::new(format!("p{i}"));
            seeding.push(p.get_id());
            players.push(p);
        }

        let _bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );

        for _p in players {
            // FIXME it takes 2 seconds per iteration to disqualify 1 player
            // let start = Instant::now();
            // if !bracket.is_over() {
            //     bracket = bracket
            //         .disqualify_participant(p.get_id())
            //         .expect("updated bracket")
            //         .0;
            // }
            // let duration = start.elapsed();
            // println!("Disqualifying took: {duration:?}");
        }

        let duration = start.elapsed();
        println!("Time elapsed in expensive_function() is: {duration:?}");
    }
}
