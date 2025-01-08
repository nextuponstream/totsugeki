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
/// A sequence of events for at most 512 players (big online bracket)
///
/// Because fuzzing thoroughly through a lot of players slows iteration speed
/// by a lot, 1023 events is chosen (for 512 players)
/// 512 exceeds default timeout (1200), so let's use less
/// 256 is good enough hopefully
pub struct BigOnlineBracketEvents(pub Vec<MatchEvent>);

#[derive(Debug)]
/// A sequence of events for 512 up to 28k players
///
/// 7k players is possible when looking at SF6 tournament at EVO 2023. This
/// might need adjustement if fighting games keep growing more popular.
///
/// For the upper bound, the we take a real expectation (7k), double it, then
/// double it again just in case.
///
/// Fuzzing thoroughly (one combination of events applied to 3, 4... 28k
/// player) in one pass will be a little too long.
pub struct StillRealisticEvents {
    /// Sequence of events
    pub sequence: Vec<MatchEvent>,
    /// Index of permutation
    pub permutation: Vec<usize>,
}

#[derive(Arbitrary, Debug, Clone, Hash, Eq, PartialEq, Copy)]
/// A match result
pub enum MatchEvent {
    /// Disqualification
    Disqualification(bool),
    /// Win reported by TO
    TOWin(bool),
}

#[derive(Debug, Copy, Clone)]
/// Format of bracket used for fuzzing
pub enum BracketFormat {
    /// Single elimination
    SingleElimination,
    /// Double elimination
    DoubleElimination,
}

impl<'a> Arbitrary<'a> for BracketFormat {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        Ok(if u.ratio(2, 10)? {
            Self::SingleElimination
        } else {
            Self::DoubleElimination
        })
    }
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
        // a valid combination for 5 events could be
        // [Disqualification(true),
        // Disqualification(false), TOWin(true), TOWin(true), TOWin(true)]
        let combination = combinations[combination_index]
            .clone()
            .into_iter()
            .map(|e| *e)
            .collect::<Vec<MatchEvent>>();

        // for 5 events, a valid permutation could be [1,3,4,2,0]
        let permutations = (0..len).permutations(len).into_iter().collect_vec();
        let permutation_index = u
            .int_in_range(0..=(permutations.len() - 1))
            .expect("permutation index");
        println!("{:?}", combination);

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

impl<'a> Arbitrary<'a> for BigOnlineBracketEvents {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        let n = 511;
        let mut events = Vec::with_capacity(n);
        for _ in 0..n {
            let element = MatchEvent::arbitrary(u)?;
            events.push(element);
        }

        Ok(BigOnlineBracketEvents(events))
    }
}

// current with like 24k, I hit 12G RAM easily
// computing permutations and combinations means consuming one of the iterator,
// which is too expensive at big player number. Then maybe creating the list of
// event, event order in "normal order", mix both list and zip is also correct
// and faster to compute while covering the whole space...
// I could limit to "one of the first 10k unique permutation" but I may miss
// some permutation.
impl<'a> Arbitrary<'a> for StillRealisticEvents {
    fn arbitrary(u: &mut Unstructured<'a>) -> Result<Self> {
        // 512 players -> 1023 events for double elimination
        // 28000 -> 55999
        let len = u.int_in_range(1023..=55999).expect("event count");
        // let len = 51;
        println!("for {} players", (len + 1) / 2);

        let types_of_events = vec![
            MatchEvent::Disqualification(true),
            MatchEvent::Disqualification(false),
            MatchEvent::TOWin(true),
            MatchEvent::TOWin(false),
        ];
        let mut events: Vec<MatchEvent> = vec![];
        // println!("choosing events...");
        for _ in 0..len {
            let event = u.choose(&types_of_events).unwrap();
            events.push(*event);
        }

        // usually, you shuffle with rand but here, our source of randomness
        // must come from the fuzzer
        // with len = 5, you see it generates stuff like [4, 2, 0, 1, 3]
        // the first permutation choices often are in sorted order [0, 1, 2...]
        // FIXME runs for too long
        // cargo +nightly fuzz run still_realistic_events -- -max_total_time=60 -timeout=60 -seed=3931501435
        // NOTE: when setting n as the number of swaps, permutation results in
        // smth like
        // [0,8848,8847,8846,8845,8844,8843,8842,8841,8840,8839,...], the
        // fuzzer attempts at resolving from the top of the tree down to the
        // end leaves of the tree. Since only the matches at the end of the
        // bracket tree can be resolved, this is really innefficient way of
        // fuzzing.
        // This results in catastrophic runtime execution because you will
        // loop through all matches ~8k times while not resolving any match.
        // Ideally, you should resolve at least one match that is ready to play
        // every time.
        // This is not an efficient way to fuzz. To salvage this, you must
        // limit the number of swap or fuzz differently.
        // TODO create function "matches_to_play" that returns a list,
        // randomize that list and resolve all those matches. Repeat that up to
        // n times (n being the number of total matches) and assert if bracket
        // is over.
        let mut choices: Vec<usize> = (0..len).collect();
        let mut permutation: Vec<usize> = vec![];
        // println!("choosing event processing order...");
        // for _ in 0..len {
        // FIXME what preserving is not how you do it?
        // always 8888, 8887, 8886 when using 0 or choices.l - 1...
        // let mut choice_index = choices.len();
        // if u.ratio(1, 10).unwrap() {
        //     choice_index = u.choose_index(choices.len()).unwrap();
        // }

        // println!("choice index: {choice_index}");
        // Note: remove is O(n) but swap_remove is O(1) without preserving
        // the order. That's fine.
        // let choice = choices.swap_remove(choice_index);
        // println!("choice: {choice}");
        // permutation.push(choice);
        // }
        // FIXME can't resolve in 40 seconds still for pb seed
        let permutation = choices;

        // Note: swap might be more efficient because O(1) in spatial
        // complexity
        // for _ in 0..100 {
        //     let index_first = u.choose_index(choices.len()).unwrap();
        //     let index_second = u.choose_index(choices.len()).unwrap();
        //     choices.swap(index_first, index_second);
        // }

        println!("{events:?}");

        assert_eq!(events.len(), permutation.len());

        let r = StillRealisticEvents {
            sequence: events,
            permutation,
        };
        // println!("{}", r.sequence.len());
        Ok(r)
    }
}
