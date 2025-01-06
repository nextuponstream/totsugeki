//! Double elimination bracket with methods you would only call on a double
//! elimination bracket

use crate::bracket::winner_bracket::winner_bracket;
use crate::bracket::PartitionError;
use crate::double_elimination_bracket::DoubleEliminationBracket;
use crate::matches::Match;

impl DoubleEliminationBracket {
    /// Partitions double elimination bracket matches in winner bracket, looser
    /// bracket, grand finals and grand finals reset for `n` players
    /// (inferred from seeding)
    ///
    /// # Errors
    /// Returns an error when there is less than 3 players in the bracket
    /// # Panics
    /// When data is corrupted
    pub fn partition_matches(
        &self,
    ) -> Result<(Vec<Match>, Vec<Match>, Match, Match), PartitionError> {
        if self.seeding.len() < 3 {
            return Err(PartitionError::NotEnoughPlayersInBracket);
        }

        let n = self.seeding.len();
        assert_eq!(
            self.matches.len(),
            2 * n - 1,
            "expected (2 * n) - 1 matches, where n is the number of players but got: {} players for {} matches",
            n,
            self.matches.len(),
        );
        let total_winner_bracket_matches = n - 1;

        // Assumes `matches` are ordered as follows: [winner bracket, loser bracket, grand final,
        // grand final reset]
        let (winner_bracket, other) = self.matches.split_at(total_winner_bracket_matches);
        let (grand_finals_reset, other) = other.split_last().expect("grand finals reset");
        let (grand_finals, loser_bracket) = other.split_last().expect("grand finals");
        Ok((
            winner_bracket.to_vec(),
            loser_bracket.to_vec(),
            *grand_finals,
            *grand_finals_reset,
        ))
    }

    /// Returns winner bracket partitionned by round
    ///
    /// # Errors
    /// When there is not enough players in the bracket for matches
    pub fn partition_winner_bracket(&self) -> Result<Vec<Vec<Match>>, PartitionError> {
        let (wb_matches, _, _, _) = self.partition_matches()?;

        Ok(winner_bracket(wb_matches, &self.seeding))
    }

    /// Returns loser bracket partitionned by round
    ///
    /// # Errors
    /// When there is not enough players in the bracket for matches
    pub fn partition_loser_bracket(&self) -> Result<Vec<Vec<Match>>, PartitionError> {
        let (_, lb_matches, _, _) = self.partition_matches()?;
        Ok(loser_bracket(lb_matches))
    }

    /// Returns Grand Finals and Grand finals reset
    ///
    /// # Errors
    /// When there is not enough players in the bracket for matches
    pub fn grand_finals_and_reset(&self) -> Result<(Match, Match), PartitionError> {
        let (_, _, gf, gf_reset) = self.partition_matches()?;
        Ok((gf, gf_reset))
    }
}

/// Partition loser brackets matches into rounds
fn loser_bracket(lb_matches: Vec<Match>) -> Vec<Vec<Match>> {
    // 2 is grand finals and grand finals reset
    let mut rounds = vec![];

    let mut matches_for_current_round = 1;
    let mut round = vec![];
    let mut round_qualifies_to_fight_next_wave_opponents = true;

    for m in lb_matches.into_iter().rev() {
        round.push(m);

        if round.len() == matches_for_current_round {
            round.reverse();
            let finalized_round = std::mem::take(&mut round);
            rounds.push(finalized_round);

            if !round_qualifies_to_fight_next_wave_opponents {
                // previous round in LB has around the same number of matches
                matches_for_current_round *= 2;
            }
            round_qualifies_to_fight_next_wave_opponents =
                !round_qualifies_to_fight_next_wave_opponents;
        }
    }

    if !round.is_empty() {
        let finalized_round = std::mem::take(&mut round);
        rounds.push(finalized_round);
    }
    rounds.reverse();

    rounds
}

#[cfg(test)]
mod tests {
    use super::PartitionError;
    use crate::bracket::seeding::Seeding;
    use crate::double_elimination_bracket::DoubleEliminationBracket;
    use crate::matches::result::MatchFormat;
    use crate::player::{Player, PlayerID};
    use crate::validation::AutomaticMatchValidationMode;

    #[test]
    fn less_than_3_participants_throws_error() {
        let bracket = DoubleEliminationBracket::default();

        let rounds = bracket.partition_loser_bracket();

        match rounds {
            Err(PartitionError::NotEnoughPlayersInBracket) => {}
            Ok(r) => panic!("expected error for 0 participants but got {r:?}"),
        }

        // 1
        let mut seeding = vec![];
        for _ in 1..=1 {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), 1);

        let rounds = bracket.partition_loser_bracket();

        match rounds {
            Err(PartitionError::NotEnoughPlayersInBracket) => {}
            Ok(r) => panic!("expected error for 1 participants but got {r:?}"),
        }

        // 2
        let mut seeding = vec![];
        for _i in 1..=2 {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), 2);

        let rounds = bracket.partition_loser_bracket();

        match rounds {
            Err(PartitionError::NotEnoughPlayersInBracket) => {}
            Ok(r) => panic!("expected error for 2 participants but got {r:?}"),
        }
    }

    #[test]
    fn _3_participants_bracket() {
        let n = 3;
        let mut seeding = vec![];
        for _i in 1..=n {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), n);

        let rounds = bracket.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 1, "expected 1 round");
        assert_eq!(rounds[0].len(), 1, "expected 1 match in round 1 LB");
        assert_eq!(rounds[0][0].get_id(), bracket.matches[2].get_id());
    }

    #[test]
    fn _4_participants_bracket() {
        let n = 4;
        let mut seeding = vec![];
        for _i in 1..=n {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), n);

        let rounds = bracket.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 2, "expected 2 round");
        assert_eq!(rounds[0].len(), 1, "expected 1 match in round 1 LB");
        assert_eq!(rounds[1].len(), 1, "expected 1 match in round 2 LB");
        assert_eq!(rounds[0][0].get_id(), bracket.matches[3].get_id(),);
        assert_eq!(rounds[1][0].get_id(), bracket.matches[4].get_id(),);
    }

    #[test]
    fn _5_participants_bracket() {
        let n = 5;
        let mut seeding = vec![];
        for _i in 1..=n {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), n);

        let rounds = bracket.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 3, "expected 3 round");
        assert_eq!(rounds[0].len(), 1, "expected 1 match in round 1 LB");
        assert_eq!(rounds[1].len(), 1, "expected 1 match in round 2 LB");
        assert_eq!(rounds[2].len(), 1, "expected 1 match in round 3 LB");
        assert_eq!(rounds[0][0].get_id(), bracket.matches[4].get_id(),);
        assert_eq!(rounds[1][0].get_id(), bracket.matches[5].get_id(),);
        assert_eq!(rounds[2][0].get_id(), bracket.matches[6].get_id(),);
    }

    #[test]
    fn _6_participants_bracket() {
        let n = 6;
        let mut seeding = vec![];
        for _i in 1..=n {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), n);

        let rounds = bracket.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 3, "expected 3 round");
        assert_eq!(rounds[0].len(), 2, "expected 2 match in round 1 LB");
        assert_eq!(rounds[1].len(), 1, "expected 1 match in round 2 LB");
        assert_eq!(rounds[2].len(), 1, "expected 1 match in round 3 LB");
        assert_eq!(
            rounds[0][0].get_id(),
            bracket.matches[5].get_id(),
            "3-6 {}",
            bracket.matches[5].summary(),
        );
        assert_eq!(rounds[0][1].get_id(), bracket.matches[6].get_id(), "4-5");
        assert_eq!(rounds[1][0].get_id(), bracket.matches[7].get_id(), "3-4");
        assert_eq!(rounds[2][0].get_id(), bracket.matches[8].get_id(), "2-3");
    }

    #[test]
    fn _7_participants_bracket() {
        let n = 7;
        let mut seeding = vec![];
        for _i in 1..=n {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), n);

        let rounds = bracket.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 4, "expected 3 round");
        assert_eq!(rounds[0].len(), 1, "expected 2 match in round 1 LB, 6-7");
        assert_eq!(
            rounds[1].len(),
            2,
            "expected 2 match in round 2 LB, 3-6 + 4-5"
        );
        assert_eq!(rounds[2].len(), 1, "expected 1 match in round 3 LB, 3-4");
        assert_eq!(rounds[3].len(), 1, "expected 1 match in round 4 LB, 2-3");
        assert_eq!(
            rounds[0][0].get_id(),
            bracket.matches[6].get_id(),
            "6-7 {}",
            bracket.matches[6].summary(),
        );
        assert_eq!(rounds[1][0].get_id(), bracket.matches[7].get_id(), "3-6");
        assert_eq!(rounds[1][1].get_id(), bracket.matches[8].get_id(), "4-5");
        assert_eq!(rounds[2][0].get_id(), bracket.matches[9].get_id(), "3-4");
        assert_eq!(rounds[3][0].get_id(), bracket.matches[10].get_id(), "2-3");
    }

    #[test]
    fn _8_participants_bracket() {
        let n = 8;
        let mut seeding = vec![];
        for _i in 1..=n {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), n);

        let rounds = bracket.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 4);
        assert_eq!(
            rounds[0].len(),
            2,
            "expected 2 match in round 1 LB, 5-8 + 6-7"
        );
        assert_eq!(
            rounds[1].len(),
            2,
            "expected 2 match in round 2 LB, 3-6 + 4-5"
        );
        assert_eq!(rounds[2].len(), 1, "expected 1 match in round 3 LB, 3-4");
        assert_eq!(rounds[3].len(), 1, "expected 1 match in round 4 LB, 2-3");
        assert_eq!(
            rounds[0][0].get_id(),
            bracket.matches[7].get_id(),
            "5-8 {}",
            bracket.matches[7].summary(),
        );
        assert_eq!(
            rounds[0][1].get_id(),
            bracket.matches[8].get_id(),
            "6-7 {}",
            bracket.matches[8].summary(),
        );
        assert_eq!(rounds[1][0].get_id(), bracket.matches[9].get_id(), "3-6");
        assert_eq!(rounds[1][1].get_id(), bracket.matches[10].get_id(), "4-5");
        assert_eq!(rounds[2][0].get_id(), bracket.matches[11].get_id(), "3-4");
        assert_eq!(rounds[3][0].get_id(), bracket.matches[12].get_id(), "2-3");
    }

    #[test]
    fn _9_participants_bracket() {
        let n = 9;
        let mut seeding = vec![];
        for _i in 1..=n {
            seeding.push(PlayerID::create());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(seeding).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );
        assert_eq!(bracket.get_seeding().len(), n);

        let rounds = bracket.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 5);
        assert_eq!(rounds[0].len(), 1, "expected 1 match in round 1 LB, 8-9");
        assert_eq!(
            rounds[1].len(),
            2,
            "expected 2 match in round 2 LB, 5-8 + 6-7"
        );
        assert_eq!(
            rounds[2].len(),
            2,
            "expected 2 match in round 3 LB, 3-6 + 4-5"
        );
        assert_eq!(rounds[3].len(), 1, "expected 1 match in round 4 LB, 3-4");
        assert_eq!(rounds[4].len(), 1, "expected 1 match in round 5 LB, 2-3");
        assert_eq!(
            rounds[0][0].get_id(),
            bracket.matches[8].get_id(),
            "5-8 {}",
            bracket.matches[8].summary(),
        );
        assert_eq!(
            rounds[1][0].get_id(),
            bracket.matches[9].get_id(),
            "5-8 {}",
            bracket.matches[9].summary(),
        );
        assert_eq!(
            rounds[1][1].get_id(),
            bracket.matches[10].get_id(),
            "6-7 {}",
            bracket.matches[10].summary(),
        );
        assert_eq!(rounds[2][0].get_id(), bracket.matches[11].get_id(), "3-6");
        assert_eq!(rounds[2][1].get_id(), bracket.matches[12].get_id(), "4-5");
        assert_eq!(rounds[3][0].get_id(), bracket.matches[13].get_id(), "3-4");
        assert_eq!(rounds[4][0].get_id(), bracket.matches[14].get_id(), "2-3");
    }

    #[test]
    fn partition_matches_for_3_man_bracket() {
        let mut player_ids = vec![PlayerID::create()]; // padding for readability
        let mut unpadded_player_ids = vec![];
        for i in 1..=3 {
            let player = Player::new(format!("p{i}"));
            player_ids.push(player.get_id());
            unpadded_player_ids.push(player.get_id());
        }
        let bracket = DoubleEliminationBracket::create(
            Seeding::new(unpadded_player_ids).unwrap(),
            AutomaticMatchValidationMode::Flexible,
            MatchFormat::ft2(),
            None,
        );

        let (winner_bracket, loser_bracket, _gf, _gfr) = bracket.partition_matches().unwrap();
        assert_eq!(winner_bracket.len(), 2);
        assert_eq!(loser_bracket.len(), 1);
        assert_eq!(loser_bracket[0].get_seeds(), [2, 3]);
    }
}
