//! Disqualification of player in bracket and all side-effects

use crate::{
    bracket::{Bracket, Error},
    matches::Match,
    player::Id as PlayerId,
};

use super::matches::is_disqualified;

/// Returns new matches when comparing old bracket and new bracket
pub(crate) fn get_new_matches(old_bracket: &[Match], new_bracket: &[Match]) -> Vec<Match> {
    new_bracket
        .iter()
        .filter(|new_m| {
            !old_bracket
                .iter()
                .any(|old_m| old_m.get_id() == new_m.get_id())
        })
        .copied()
        .collect::<Vec<Match>>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bracket::seeding::Seeding;
    use crate::double_elimination_bracket::DoubleEliminationBracket;
    use crate::player::Participants;
    use crate::validation::AutomaticMatchValidationMode;
    use crate::{format::Format, player::Player};
    use std::time::Instant;

    // cargo t disqualify_8000 -- --include-ignored --nocapture
    #[test]
    #[ignore]
    fn disqualify_8000_player_bracket() {
        let start = Instant::now();
        let participants = Participants::default();
        let mut players = vec![];
        let mut seeding = vec![];

        for i in 1..=8000 {
            let p = Player::new(format!("p{i}"));
            seeding.push(p.get_id());
            players.push(p);
        }

        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
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
