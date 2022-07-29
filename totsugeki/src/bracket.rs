//! Bracket domain

use crate::{
    matches::Match,
    organiser::Id as OrganiserId,
    player::{Id as PlayerId, Players},
    seeding::{
        get_balanced_round_matches_top_seed_favored, seed, Error as SeedingError,
        Method as SeedingMethod,
    },
    DiscussionChannelId,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use uuid::Uuid;

/// Error while manipulating brackets
#[derive(Debug)]
pub enum Error {
    /// Error while seeding a bracket
    Seeding(SeedingError),
    /// Missing argument
    MissingArgument,
}

/// All bracket formats
#[derive(PartialEq, Eq, Clone, Deserialize, Serialize, Debug)]
pub enum Format {
    /// Single elimination tournament
    SingleElimination,
    // TODO add other style of tournament
}

impl Default for Format {
    fn default() -> Self {
        Self::SingleElimination // TODO set to DoubleElimination when implemented
    }
}

/// Active brackets
pub type ActiveBrackets = HashMap<DiscussionChannelId, Id>;

/// Finalized brackets
pub type FinalizedBrackets = HashSet<Id>;

#[derive(Serialize, Deserialize)]
/// POST request to /bracket endpoint
pub struct POST {
    /// name of the bracket
    bracket_name: String,
    /// used to create missing organiser
    organiser_name: String,
    organiser_internal_id: String,
    channel_internal_id: String,
    service_type_id: String,
}

impl POST {
    /// Create new Bracket POST request
    #[must_use]
    pub fn new(
        bracket_name: String,
        organiser_name: String,
        organiser_internal_id: String,
        channel_internal_id: String,
        service_type_id: String,
    ) -> Self {
        POST {
            bracket_name,
            organiser_name,
            organiser_internal_id,
            channel_internal_id,
            service_type_id,
        }
    }
}

/// Bracket for a tournament
///
/// Seeding is important: <https://youtu.be/ZGoIIV55hEc?t=108>
///
/// TLDR: do not match good players against each other early on so good players
/// don't end up below players they would have beaten otherwise
// TODO add factor to not match local players against each other
// idea: 1st and 2nd player get placed separately, then try to avoid matching
// any two players from the same local for the first round at least. What you
// would really want is to put players from the same local as far away from
// each of them as possible
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Bracket {
    bracket_id: Id,
    bracket_name: String,
    players: Vec<PlayerId>,
    is_seeded: bool,
    matches: Vec<Vec<Match>>,
    format: Format,
    seeding_method: SeedingMethod,
}

impl Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{{ bracket_id: {}, bracket_name \"{} \"}}",
            self.bracket_id, self.bracket_name
        )
    }
}

/// A collection of brackets
#[derive(Default)]
pub struct Brackets {
    brackets: Vec<Bracket>,
}

impl Brackets {
    /// Create representation of brackets implementing `std::fmt::Display`
    #[must_use]
    pub fn new(brackets: Vec<Bracket>) -> Self {
        Brackets { brackets }
    }

    /// Get brackets
    #[must_use]
    pub fn get_brackets(&self) -> Vec<Bracket> {
        self.brackets.clone()
    }
}

impl Display for Brackets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for b in self.brackets.clone() {
            b.fmt(f)?;
        }
        Ok(())
    }
}

impl Bracket {
    /// Create new bracket
    #[must_use]
    pub fn new(
        bracket_name: String,
        players: Vec<PlayerId>,
        format: Format,
        seeding_method: SeedingMethod,
    ) -> Self {
        // TODO add check where registration_start_time < beginning_start_time
        Bracket {
            bracket_id: Uuid::new_v4(),
            bracket_name,
            players,
            is_seeded: false,
            matches: vec![],
            format,
            seeding_method,
        }
    }

    /// Create from existing bracket
    #[must_use]
    pub fn from(
        id: Id,
        bracket_name: String,
        players: Vec<PlayerId>,
        format: Format,
        seeding_method: SeedingMethod,
    ) -> Self {
        Self {
            bracket_id: id,
            bracket_name,
            players,
            is_seeded: false,
            matches: vec![],
            format,
            seeding_method,
        }
    }

    /// Get ID of bracket
    #[must_use]
    pub fn get_id(&self) -> Uuid {
        self.bracket_id
    }

    /// Get name of bracket
    #[must_use]
    pub fn get_bracket_name(&self) -> String {
        self.bracket_name.clone()
    }

    /// Get players
    #[must_use]
    pub fn get_players(&self) -> Vec<PlayerId> {
        self.players.clone()
    }

    /// Seed bracket
    ///
    /// # Errors
    /// Returns an error if necessary arguments for seeding are missing
    pub fn seed(self, players: Option<Players>) -> Result<Bracket, Error> {
        let matches: Vec<Vec<Match>> = match self.format {
            Format::SingleElimination => match players {
                Some(players) => {
                    let players = seed(&self.seeding_method, players)?;
                    get_balanced_round_matches_top_seed_favored(&players)
                }
                None => return Err(Error::MissingArgument),
            },
        };
        let mut bracket = self;
        bracket.matches = matches;
        Ok(bracket)
    }

    /// Get matches
    #[must_use]
    pub fn get_matches(&self) -> Vec<Vec<Match>> {
        self.matches.clone()
    }
}

impl From<SeedingError> for Error {
    fn from(e: SeedingError) -> Self {
        Self::Seeding(e)
    }
}

/// Bracket identifier
pub type Id = Uuid;

/// POST response to /bracket endpoint
#[derive(Serialize, Deserialize)]
pub struct POSTResult {
    /// id of created bracket
    bracket_id: Id,
    /// id of organiser
    organiser_id: OrganiserId,
    /// id of discussion channel
    discussion_channel_id: DiscussionChannelId,
}

impl POSTResult {
    #[must_use]
    /// Create new bracket from values
    pub fn from(
        bracket_id: Id,
        organiser_id: Id,
        discussion_channel_id: DiscussionChannelId,
    ) -> Self {
        Self {
            bracket_id,
            organiser_id,
            discussion_channel_id,
        }
    }

    #[must_use]
    /// Get bracket id
    pub fn get_bracket_id(&self) -> Id {
        self.bracket_id
    }

    #[must_use]
    /// Get organiser id
    pub fn get_organiser_id(&self) -> Id {
        self.organiser_id
    }

    #[must_use]
    /// Get discussion channel id
    pub fn get_discussion_channel_id(&self) -> DiscussionChannelId {
        self.discussion_channel_id
    }
}
