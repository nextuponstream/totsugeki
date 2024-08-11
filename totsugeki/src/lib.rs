#![deny(missing_docs)]
#![deny(clippy::missing_docs_in_private_items)]
#![deny(rustdoc::invalid_codeblock_attributes)]
#![warn(rustdoc::bare_urls)]
#![deny(rustdoc::broken_intra_doc_links)]
#![doc = include_str!("../../README.md")]
#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(clippy::unwrap_used)]
#![forbid(unsafe_code)]

use uuid::Uuid;

pub mod bracket;
pub mod double_elimination_bracket;
pub mod format;
pub mod matches;
pub mod opponent;
pub mod player;
pub mod seeding;
pub mod single_elimination_bracket;

/// ID for bracket, players...
pub type ID = Uuid;

#[cfg(test)]
/// Helper function for test cases with small group of players. Instead of
/// using Player struct for p1, p2... p9, you can use made up identifiable ids
/// like:
///
/// * 00..01 for player 1
/// * 00..02 for player 2
/// * ... and so on
///
/// # Panics
/// when n is not between 1 and 16
pub(crate) fn legible_uuids_order(n: usize) -> Vec<Uuid> {
    assert_ne!(n, 0, "This function cannot return an empty vector");
    assert!(
        n <= 16,
        "This function does not accept number greater than 9"
    );

    let mut r = vec![];

    for i in 1..=n {
        let id = match i {
            small if small <= 9 => format!("{i:02}"),
            10 => "0A".into(),
            11 => "0B".into(),
            12 => "0C".into(),
            13 => "0D".into(),
            14 => "0E".into(),
            15 => "0F".into(),
            16 => "10".into(),
            _ => unreachable!(),
        };
        let p = format!("00000000-0000-0000-0000-0000000000{id}");
        r.push(p.parse::<crate::player::Id>().expect("id"));
    }

    r
}

#[cfg(test)]
mod tests {
    use crate::legible_uuids_order;

    #[test]
    fn made_up_uuids_work_in_accepted_range() {
        for i in 1..=16 {
            let _uuids = legible_uuids_order(i);
        }
    }
}
