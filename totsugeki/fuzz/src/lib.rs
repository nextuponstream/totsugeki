//! Fuzzing utilities

#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]

use arbitrary::{Arbitrary, Result, Unstructured};
use itertools::Itertools;

#[derive(Debug, Arbitrary)]
/// A sequence of events
pub struct Events {
    /// Sequence of events
    pub sequence: Vec<MatchEvent>,
}

#[derive(Debug)]
/// A unique permutation of events
pub struct EventsPermutation {
    /// Sequence of events
    pub sequence: Vec<MatchEvent>,
    /// Index of permutation
    pub permutation: usize,
}

#[derive(Debug)]
/// A sequence of events for at most 65 players
///
/// Because fuzzing thoroughly through a lot of players slows iteration speed
/// by a lot, 129 events is chosen (for 65 players)
pub struct LotsOfEvents(pub Vec<MatchEvent>);

#[derive(Debug)]
/// A sequence of events for at most 2100 players
///
/// Because fuzzing thoroughly through a lot of players slows iteration speed
/// by a lot, 129 events is chosen (for 65 players)
pub struct ExtremeLotsOfEvents(pub Vec<MatchEvent>);

#[derive(Arbitrary, Debug, Clone, Hash, Eq, PartialEq, Copy)]
/// A match result
pub enum MatchEvent {
    /// Disqualification
    Disqualification(bool),
    // NOTE: for exhaustive testing, you can uncomment this but this makes it
    // way longer
    /// Win reported by both players
    // Win(bool),
    /// Win reported by TO
    TOWin(bool),
}

#[derive(Arbitrary, Debug, Copy, Clone)]
/// Format of bracket used for fuzzing
pub enum BracketFormat {
    /// Single elimination
    SingleElimination,
    /// Double elimination
    DoubleElimination,
}

impl<'a> Arbitrary<'a> for EventsPermutation {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let len = match u.arbitrary_len::<MatchEvent>()? {
            l if l == 0 => 1,
            l => l,
        };

        let types_of_events = vec![
            MatchEvent::Disqualification(true),
            MatchEvent::Disqualification(false),
            MatchEvent::TOWin(true),
            MatchEvent::TOWin(false),
        ];
        let combinations = types_of_events
            .iter()
            .combinations_with_replacement(len)
            .collect_vec();
        let combination_index = u
            .int_in_range(0..=(combinations.len() - 1))
            .expect("combination index");
        let combination = combinations[combination_index]
            .iter()
            .map(|e| *e.clone())
            .collect::<Vec<MatchEvent>>();

        let permutations = (0..len).permutations(len).into_iter().collect_vec();
        let permutation_index = u
            .int_in_range(0..=(permutations.len() - 1))
            .expect("permutation index");

        Ok(EventsPermutation {
            sequence: combination.clone(),
            permutation: permutation_index,
        })
    }
}

impl<'a> Arbitrary<'a> for LotsOfEvents {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let n = 129;
        let mut events = Vec::with_capacity(n);
        for _ in 0..n {
            let element = MatchEvent::arbitrary(u)?;
            events.push(element);
        }

        Ok(LotsOfEvents(events))
    }
}

impl<'a> Arbitrary<'a> for ExtremeLotsOfEvents {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let n = 4199;
        let mut events = Vec::with_capacity(n);
        for _ in 0..n {
            let element = MatchEvent::arbitrary(u)?;
            events.push(element);
        }

        Ok(ExtremeLotsOfEvents(events))
    }
}
