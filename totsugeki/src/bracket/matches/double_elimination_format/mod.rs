//! Manage matches from double elimination bracket

use super::{
    assert_disqualified_at_most_once, assert_match_is_well_formed, update_bracket_with, Error,
    Progression,
};
use crate::bracket::seeding::Seeding;
use crate::{
    bracket::{
        disqualification::get_new_matches,
        progression::{new_matches_to_play_for_bracket, winner_of_bracket},
    },
    matches::{
        double_elimination_matches_from_partition as dem_partition,
        partition_double_elimination_matches, Error as MatchError, Id as MatchId, Match,
        ReportedResult,
    },
    opponent::Opponent,
    player::Id as PlayerId,
    seeding::{
        double_elimination_seeded_bracket::get_loser_bracket_matches_top_seed_favored,
        single_elimination_seeded_bracket::get_balanced_round_matches_top_seed_favored,
    },
    ID,
};

/// Set player as disqualified. Used when there is no need for further updates

#[cfg(test)]
mod tests {}
