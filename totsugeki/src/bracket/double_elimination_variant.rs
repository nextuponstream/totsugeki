//! Double elimination bracket variant with methods you would only call on a
//! double elimination bracket

use super::winner_bracket::winner_bracket;
use crate::bracket::Bracket;
use crate::bracket::PartitionError;
use crate::format::Format;
use crate::matches::partition_double_elimination_matches as partition;
use crate::matches::Match;

/// Double elimination bracket
#[derive(Debug)]
pub struct Variant {
    /// Some bracket
    bracket: Bracket,
}

/// Error with double elimination brackets
#[derive(Debug)]
pub enum TryIntoError {
    /// Expected format to be double-elimination
    ExpectedDoubleEliminationFormat,
}

impl TryFrom<Bracket> for Variant {
    type Error = TryIntoError;

    // TODO try to use type state pattern instead of throwing errors
    // trying to coerce smth into smth else is a sign that a fallible process
    // should be made not fallible
    fn try_from(bracket: Bracket) -> Result<Self, Self::Error> {
        if bracket.format != Format::DoubleElimination {
            return Err(TryIntoError::ExpectedDoubleEliminationFormat);
        }

        Ok(Variant { bracket })
    }
}

impl std::default::Default for Variant {
    fn default() -> Self {
        let bracket = Bracket {
            format: Format::DoubleElimination,
            ..Default::default()
        };
        Self { bracket }
    }
}

impl Variant {
    /// Returns winner bracket, loser bracket, grand finals and grand final reset
    ///
    /// # Errors
    /// Returns an error when there is less than 3 players in the bracket
    pub fn partition_matches(
        &self,
    ) -> Result<(Vec<Match>, Vec<Match>, Match, Match), PartitionError> {
        if self.bracket.participants.len() < 3 {
            return Err(PartitionError::NotEnoughPlayersInBracket);
        }
        Ok(partition(
            &self.bracket.matches,
            self.bracket.participants.len(),
        ))
    }

    /// Returns winner bracket partitioned by round
    ///
    /// # Errors
    /// When there is not enough players in the bracket for matches
    #[must_use]
    pub fn partition_winner_bracket(&self) -> Option<Vec<Vec<Match>>> {
        if self.bracket.participants.len() < 3 {
            return None;
        }
        let (wb_matches, _, _, _) =
            partition(&self.bracket.matches, self.bracket.participants.len());

        Some(winner_bracket(wb_matches, &self.bracket.participants))
    }

    /// Returns loser bracket partitioned by round
    ///
    /// # Errors
    /// When there is not enough players in the bracket for matches
    #[must_use]
    pub fn partition_loser_bracket(&self) -> Option<Vec<Vec<Match>>> {
        if self.bracket.participants.len() < 3 {
            return None;
        }
        let (_, lb_matches, _, _) =
            partition(&self.bracket.matches, self.bracket.participants.len());
        Some(loser_bracket(lb_matches))
    }

    /// Returns Grand Finals and Grand finals reset
    ///
    /// # Errors
    /// When there is not enough players in the bracket for matches
    #[must_use]
    pub fn grand_finals_and_reset(&self) -> Option<(Match, Match)> {
        if self.bracket.participants.len() < 3 {
            return None;
        }
        let (_, _, gf, gf_reset) =
            partition(&self.bracket.matches, self.bracket.participants.len());
        Some((gf, gf_reset))
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
    use crate::bracket::double_elimination_variant::Variant;

    #[test]
    fn less_than_3_participants_returns_empty() {
        let deb = Variant::default();

        let rounds = deb.partition_loser_bracket();

        assert!(rounds.is_none());
        // 1
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        for i in 1..=1 {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };
        assert_eq!(deb.bracket.participants.len(), 1);

        let rounds = deb.partition_loser_bracket();

        assert!(rounds.is_none());

        // 2
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        for i in 1..=2 {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };
        assert_eq!(deb.bracket.participants.len(), 2);

        let rounds = deb.partition_loser_bracket();

        assert!(rounds.is_none());
    }

    #[test]
    fn _3_participants_bracket() {
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        let n = 3;
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };

        assert_eq!(deb.bracket.participants.len(), n);

        let rounds = deb.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 1, "expected 1 round");
        assert_eq!(rounds[0].len(), 1, "expected 1 match in round 1 LB");
        assert_eq!(rounds[0][0].get_id(), deb.bracket.matches[2].get_id());
    }

    #[test]
    fn _4_participants_bracket() {
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        let n = 4;
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };

        assert_eq!(deb.bracket.participants.len(), n);

        let rounds = deb.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 2, "expected 2 round");
        assert_eq!(rounds[0].len(), 1, "expected 1 match in round 1 LB");
        assert_eq!(rounds[1].len(), 1, "expected 1 match in round 2 LB");
        assert_eq!(rounds[0][0].get_id(), deb.bracket.matches[3].get_id(),);
        assert_eq!(rounds[1][0].get_id(), deb.bracket.matches[4].get_id(),);
    }

    #[test]
    fn _5_participants_bracket() {
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        let n = 5;
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };

        assert_eq!(deb.bracket.participants.len(), n);

        let rounds = deb.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 3, "expected 3 round");
        assert_eq!(rounds[0].len(), 1, "expected 1 match in round 1 LB");
        assert_eq!(rounds[1].len(), 1, "expected 1 match in round 2 LB");
        assert_eq!(rounds[2].len(), 1, "expected 1 match in round 3 LB");
        assert_eq!(rounds[0][0].get_id(), deb.bracket.matches[4].get_id(),);
        assert_eq!(rounds[1][0].get_id(), deb.bracket.matches[5].get_id(),);
        assert_eq!(rounds[2][0].get_id(), deb.bracket.matches[6].get_id(),);
    }

    #[test]
    fn _6_participants_bracket() {
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        let n = 6;
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };

        assert_eq!(deb.bracket.participants.len(), n);

        let rounds = deb.partition_loser_bracket().expect("partition");

        assert_eq!(rounds.len(), 3, "expected 3 round");
        assert_eq!(rounds[0].len(), 2, "expected 2 match in round 1 LB");
        assert_eq!(rounds[1].len(), 1, "expected 1 match in round 2 LB");
        assert_eq!(rounds[2].len(), 1, "expected 1 match in round 3 LB");
        assert_eq!(
            rounds[0][0].get_id(),
            deb.bracket.matches[5].get_id(),
            "3-6 {}",
            deb.bracket.matches[5].summary(),
        );
        assert_eq!(
            rounds[0][1].get_id(),
            deb.bracket.matches[6].get_id(),
            "4-5"
        );
        assert_eq!(
            rounds[1][0].get_id(),
            deb.bracket.matches[7].get_id(),
            "3-4"
        );
        assert_eq!(
            rounds[2][0].get_id(),
            deb.bracket.matches[8].get_id(),
            "2-3"
        );
    }

    #[test]
    fn _7_participants_bracket() {
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        let n = 7;
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };

        assert_eq!(deb.bracket.participants.len(), n);

        let rounds = deb.partition_loser_bracket().expect("partition");

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
            deb.bracket.matches[6].get_id(),
            "6-7 {}",
            deb.bracket.matches[6].summary(),
        );
        assert_eq!(
            rounds[1][0].get_id(),
            deb.bracket.matches[7].get_id(),
            "3-6"
        );
        assert_eq!(
            rounds[1][1].get_id(),
            deb.bracket.matches[8].get_id(),
            "4-5"
        );
        assert_eq!(
            rounds[2][0].get_id(),
            deb.bracket.matches[9].get_id(),
            "3-4"
        );
        assert_eq!(
            rounds[3][0].get_id(),
            deb.bracket.matches[10].get_id(),
            "2-3"
        );
    }

    #[test]
    fn _8_participants_bracket() {
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        let n = 8;
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };

        assert_eq!(deb.bracket.participants.len(), n);

        let rounds = deb.partition_loser_bracket().expect("partition");

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
            deb.bracket.matches[7].get_id(),
            "5-8 {}",
            deb.bracket.matches[7].summary(),
        );
        assert_eq!(
            rounds[0][1].get_id(),
            deb.bracket.matches[8].get_id(),
            "6-7 {}",
            deb.bracket.matches[8].summary(),
        );
        assert_eq!(
            rounds[1][0].get_id(),
            deb.bracket.matches[9].get_id(),
            "3-6"
        );
        assert_eq!(
            rounds[1][1].get_id(),
            deb.bracket.matches[10].get_id(),
            "4-5"
        );
        assert_eq!(
            rounds[2][0].get_id(),
            deb.bracket.matches[11].get_id(),
            "3-4"
        );
        assert_eq!(
            rounds[3][0].get_id(),
            deb.bracket.matches[12].get_id(),
            "2-3"
        );
    }

    #[test]
    fn _9_participants_bracket() {
        let deb = Variant::default();
        let mut bracket = deb.bracket;
        let n = 9;
        for i in 1..=n {
            bracket = bracket
                .add_participant(format!("p{i}").as_str())
                .expect("player added")
                .0;
        }
        let deb = Variant { bracket };

        assert_eq!(deb.bracket.participants.len(), n);

        let rounds = deb.partition_loser_bracket().expect("partition");

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
            deb.bracket.matches[8].get_id(),
            "5-8 {}",
            deb.bracket.matches[8].summary(),
        );
        assert_eq!(
            rounds[1][0].get_id(),
            deb.bracket.matches[9].get_id(),
            "5-8 {}",
            deb.bracket.matches[9].summary(),
        );
        assert_eq!(
            rounds[1][1].get_id(),
            deb.bracket.matches[10].get_id(),
            "6-7 {}",
            deb.bracket.matches[10].summary(),
        );
        assert_eq!(
            rounds[2][0].get_id(),
            deb.bracket.matches[11].get_id(),
            "3-6"
        );
        assert_eq!(
            rounds[2][1].get_id(),
            deb.bracket.matches[12].get_id(),
            "4-5"
        );
        assert_eq!(
            rounds[3][0].get_id(),
            deb.bracket.matches[13].get_id(),
            "3-4"
        );
        assert_eq!(
            rounds[4][0].get_id(),
            deb.bracket.matches[14].get_id(),
            "2-3"
        );
    }
}
