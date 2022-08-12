//! Bracket domain

use crate::{
    bracket::Id as BracketId,
    matches::{Match, MatchGET, MatchParsingError},
    organiser::Id as OrganiserId,
    player::{Player, Players},
    seeding::{
        get_balanced_round_matches_top_seed_favored, seed, Error as SeedingError,
        Method as SeedingMethod, ParsingError as SeedingParsingError,
    },
    DiscussionChannel, DiscussionChannelId,
};
use chrono::prelude::*;
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
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
#[derive(PartialEq, Eq, Copy, Clone, Deserialize, Serialize, Debug)]
pub enum Format {
    /// Single elimination tournament
    SingleElimination,
    // TODO add other style of tournament
}

impl std::fmt::Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::SingleElimination => write!(f, "single-elimination"),
        }
    }
}

/// Parsing error for Format type
#[derive(Debug)]
pub enum FormatParsingError {
    /// Unknown format was provided
    Unknown(String),
}

impl std::fmt::Display for FormatParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatParsingError::Unknown(format) => writeln!(
                f,
                "Unknown bracket format: \"{format}\". Please try another format such as: \"{}\"",
                Format::default()
            ),
        }
    }
}

impl std::str::FromStr for Format {
    type Err = FormatParsingError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "single-elimination" => Ok(Format::SingleElimination),
            _ => Err(FormatParsingError::Unknown(s.to_string())),
        }
    }
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

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
/// POST request to /bracket endpoint
pub struct POST {
    /// name of the bracket
    pub bracket_name: String,
    /// used to create missing organiser
    pub organiser_name: String,
    /// Identifier of the organiser from the service (for instance: discord)
    pub organiser_internal_id: String,
    /// Identifier of the discussion channel from service (for instance: discord)
    pub channel_internal_id: String,
    /// Name of service. See totsugeki_api for a list of supported service
    pub service_type_id: String,
    /// bracket format
    pub format: String,
    /// seeding method for bracket
    pub seeding_method: String,
    /// Advertised start time
    pub start_time: String,
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
    /// Identifier of this bracket
    bracket_id: Id,
    /// Name of this bracket
    bracket_name: String,
    /// Players of this bracket
    players: Vec<Player>,
    /// Matches from this bracket, sorted by rounds
    pub matches: Vec<Vec<Match>>,
    /// Bracket format
    format: Format,
    /// Seeding method used for this bracket
    seeding_method: SeedingMethod,
    /// Advertised start time
    start_time: DateTime<Utc>,
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
    /// A collection of brackets
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
        players: Vec<Player>,
        format: Format,
        seeding_method: SeedingMethod,
        start_time: DateTime<Utc>,
    ) -> Self {
        // TODO add check where registration_start_time < beginning_start_time
        Bracket {
            bracket_id: Uuid::new_v4(),
            bracket_name,
            players,
            matches: vec![],
            format,
            seeding_method,
            start_time,
        }
    }

    /// Create from existing bracket
    #[must_use]
    pub fn from(
        bracket_id: Id,
        bracket_name: String,
        players: Vec<Player>,
        matches: Vec<Vec<Match>>,
        format: Format,
        seeding_method: SeedingMethod,
        start_time: DateTime<Utc>,
    ) -> Self {
        Self {
            bracket_id,
            bracket_name,
            players,
            matches,
            format,
            seeding_method,
            start_time,
        }
    }

    /// Get ID of bracket
    #[must_use]
    pub fn get_id(&self) -> BracketId {
        self.bracket_id
    }

    /// Get name of bracket
    #[must_use]
    pub fn get_bracket_name(&self) -> String {
        self.bracket_name.clone()
    }

    /// Get players
    #[must_use]
    pub fn get_players(&self) -> Vec<Player> {
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
                    get_balanced_round_matches_top_seed_favored(&players)?
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

    /// Get bracket format
    #[must_use]
    pub fn get_format(&self) -> Format {
        self.format
    }

    /// Get bracket seeding method
    #[must_use]
    pub fn get_seeding_method(&self) -> SeedingMethod {
        self.seeding_method
    }

    /// Get start time
    #[must_use]
    pub fn get_start_time(&self) -> DateTime<Utc> {
        self.start_time
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
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct POSTResult {
    /// id of created bracket
    pub bracket_id: Id,
    /// id of organiser
    pub organiser_id: OrganiserId,
    /// id of discussion channel
    pub discussion_channel_id: DiscussionChannelId,
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

/// Bracket GET response
#[derive(Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct GET {
    /// Identifier of bracket
    pub bracket_id: Id,
    /// Name of this bracket
    pub bracket_name: String,
    /// Players in this bracket
    pub players: Vec<Player>,
    /// Matches for this bracket
    pub matches: Vec<Vec<MatchGET>>,
    /// Bracket format
    pub format: String,
    /// Seeding method used for this bracket
    pub seeding_method: String,
    /// Advertised start time
    pub start_time: String,
}

impl GET {
    /// Form values to be sent to the API to create a bracket
    #[must_use]
    pub fn new(bracket: &Bracket) -> Self {
        GET {
            bracket_id: bracket.get_id(),
            bracket_name: bracket.get_bracket_name(),
            players: bracket.get_players(),
            format: bracket.get_format().to_string(),
            seeding_method: bracket.get_seeding_method().to_string(),
            matches: Match::get_sendable_matches(&bracket.get_matches()),
            start_time: bracket.start_time.to_string(),
        }
    }
}

/// Error while parsing Bracket
#[derive(Debug)]
pub enum ParsingError {
    /// Could not parse bracket format
    Format(FormatParsingError),
    /// Could not parse seeding method
    Seeding(SeedingParsingError),
    /// Could not parse match
    Match(MatchParsingError),
    /// Could not parse time
    Time(chrono::ParseError),
}

impl std::error::Error for ParsingError {}

impl std::fmt::Display for ParsingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingError::Format(e) => e.fmt(f),
            ParsingError::Seeding(e) => e.fmt(f),
            ParsingError::Match(e) => e.fmt(f),
            ParsingError::Time(e) => e.fmt(f),
        }
    }
}

impl TryFrom<GET> for Bracket {
    type Error = ParsingError;

    fn try_from(b: GET) -> Result<Self, Self::Error> {
        Ok(Self {
            bracket_id: b.bracket_id,
            bracket_name: b.bracket_name,
            players: b.players,
            matches: {
                b.matches
                    .iter()
                    .map(|r| {
                        r.iter()
                            .map(|m| Match::try_from(m.clone()))
                            .collect::<Result<Vec<Match>, _>>()
                    })
                    .collect::<Result<Vec<Vec<Match>>, _>>()?
            },
            format: b.format.parse::<Format>()?,
            seeding_method: b.seeding_method.parse::<SeedingMethod>()?,
            start_time: b.start_time.parse::<DateTime<Utc>>()?,
        })
    }
}

impl From<MatchParsingError> for ParsingError {
    fn from(e: MatchParsingError) -> Self {
        Self::Match(e)
    }
}

impl From<FormatParsingError> for ParsingError {
    fn from(e: FormatParsingError) -> Self {
        Self::Format(e)
    }
}

impl From<SeedingParsingError> for ParsingError {
    fn from(e: SeedingParsingError) -> Self {
        ParsingError::Seeding(e)
    }
}

/// bracket creation request parameters
#[derive(Debug)]
pub struct RequestParameters<'a, T: DiscussionChannel> {
    /// requested bracket name
    pub bracket_name: &'a str,
    /// requested bracket format
    pub bracket_format: &'a str,
    /// organiser name of requested bracket
    pub organiser_name: &'a str,
    /// Organiser id of requested bracket
    pub organiser_id: &'a str,
    /// Discussion channel where bracket request comes from
    pub discussion_channel: T,
    /// seeding method for bracket
    pub seeding_method: &'a str,
    /// Advertised start time (UTC)
    pub start_time: &'a str,
}

impl From<Bracket> for GET {
    fn from(b: Bracket) -> Self {
        GET::new(&b)
    }
}

impl From<chrono::ParseError> for ParsingError {
    fn from(e: chrono::ParseError) -> Self {
        Self::Time(e)
    }
}
