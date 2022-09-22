// Common assertion for matches

use totsugeki::{matches::Match, player::Player};

/// Assert if for each seeding, there is a match with a corresponding seedingo
pub fn assert_seeding(expected_seedings: &[[usize; 2]], matches: &[Match]) {
    for expected_seeding in expected_seedings {
        assert!(
            matches.iter().any(|m| m.get_seeds() == *expected_seeding),
            "expected match with {expected_seeding:?} but found none"
        );
    }
}

/// Assert if match between two participants exists in collection
pub fn assert_if_match_exists(
    expected_seeding: [usize; 2],
    participants: [Player; 2],
    matches: &[Match],
) {
    assert!(
        matches.iter().any(|m| m.contains(participants[0].get_id())
            && m.contains(participants[1].get_id())
            && m.get_seeds() == expected_seeding),
        "expected match with seeding {expected_seeding:?} with {} and {}.\nExpected participants: {:?}\nGot:\n{matches:?}",
        participants[0].get_name(),
        participants[1].get_name(),
        participants
     )
}

/// Assert if matches exist in collection
pub fn assert_if_matches_exist(expected_matches: &[([usize; 2], [Player; 2])], matches: &[Match]) {
    for (expected_seeds, expected_participants) in expected_matches {
        assert_if_match_exists(*expected_seeds, expected_participants.clone(), matches);
    }
}
