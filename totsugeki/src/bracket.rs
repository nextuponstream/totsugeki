//! Bracket domain

use crate::{
    bracket::Id as BracketId,
    matches::{
        Error as MatchError, Id as MatchId, Match, MatchGET, MatchParsingError, ReportedResult,
    },
    organiser::Id as OrganiserId,
    player::{Error as PlayerError, Id as PlayerId, Player, Players},
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

/// Updating bracket cannot be performed
#[derive(Debug)]
pub enum Error {
    /// Error while seeding a bracket
    Seeding(SeedingError),
    /// Missing argument (name of the argument missing and additionnal context)
    MissingArgument(String, String),
    /// Error while updating players of bracket
    PlayerUpdate(PlayerError),
    /// Match referred does not exist for this bracket
    UnknownMatch(MatchId),
    /// Match cannot be updated
    Match(MatchError),
    /// Unknown player provided for seeding
    UnknownPlayer(PlayerId, Players),
    /// Provided players for seeding do not match players in bracket
    Players(Players, Players),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Seeding(e) => e.fmt(f),
            Error::MissingArgument(missing_argument, additionnal_context) => writeln!(
                f,
                "Missing argument: \"{missing_argument}\"\n{additionnal_context}"
            ),
            Error::PlayerUpdate(e) => e.fmt(f),
            Error::UnknownMatch(id) => {
                write!(f, "Match with id \"{id}\" does not exists in bracket.")
            }
            Error::Match(e) => e.fmt(f),
            Error::UnknownPlayer(id, players) => write!(f, "Unknow player \"{id}\" cannot be used for seeding. Use the following players: {players}"),
            Error::Players(provided, actual) => write!(f, "Provided players cannot be used: {provided}\nUse the following players: {actual}"),
        }
    }
}

impl From<MatchError> for Error {
    fn from(e: MatchError) -> Self {
        Self::Match(e)
    }
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
    /// Automatically validate match if both players agree
    pub automatic_match_validation: bool,
}

/// Raw data of bracket, potentially malformed. Use `Bracket` for well-formed bracket
#[derive(Debug, PartialEq, Eq, Default, Serialize, Deserialize, Clone)]
pub struct Raw {
    /// Identifier of this bracket
    pub bracket_id: Id,
    /// Name of this bracket
    pub bracket_name: String,
    /// Players in this bracket
    pub players: Vec<PlayerId>,
    /// Names of players in this bracket
    pub player_names: Vec<String>,
    /// Matches from this bracket, sorted by rounds
    pub matches: Vec<Match>,
    /// Bracket format
    pub format: Format,
    /// Seeding method used for this bracket
    pub seeding_method: SeedingMethod,
    /// Advertised start time
    pub start_time: DateTime<Utc>,
    /// Accept match results
    pub accept_match_results: bool,
    /// Matches are automatically validated if both players agree on result
    pub automatic_match_validation: bool,
    /// Bar new participants from entering bracket
    pub barred_from_entering: bool,
}

/// Bracket for tournament
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
#[derive(Clone)]
pub struct Bracket {
    /// Identifier of this bracket
    bracket_id: Id,
    /// Name of this bracket
    bracket_name: String,
    /// Players of this bracket
    players: Players,
    /// Matches from this bracket, sorted by rounds
    matches: Vec<Match>,
    /// Bracket format
    format: Format,
    /// Seeding method used for this bracket
    seeding_method: SeedingMethod,
    /// Advertised start time
    start_time: DateTime<Utc>,
    /// Accept match results
    accept_match_results: bool,
    /// Matches are automatically validated if both players agree on result
    automatic_match_validation: bool,
    /// Bar new participants from entering bracket
    barred_from_entering: bool,
}

impl Bracket {
    /// Create new bracket
    #[must_use]
    pub fn new(
        name: &str,
        format: Format,
        seeding_method: SeedingMethod,
        start_time: DateTime<Utc>,
        automatic_match_validation: bool,
    ) -> Self {
        Self {
            bracket_id: BracketId::new_v4(),
            bracket_name: name.to_string(),
            players: Players::default(),
            matches: vec![],
            format,
            seeding_method,
            start_time,
            accept_match_results: false,
            automatic_match_validation,
            barred_from_entering: false,
        }
    }

    /// Get id of bracket
    #[must_use]
    pub fn get_id(&self) -> Id {
        self.bracket_id
    }

    /// Get name of bracket
    #[must_use]
    pub fn get_name(&self) -> String {
        self.bracket_name.clone()
    }

    /// Get players in bracket
    #[must_use]
    pub fn get_players(&self) -> Players {
        self.players.clone()
    }

    /// Update seeding with reordered list of players and generate matches
    ///
    /// # Errors
    /// Returns an error if provided players do not match current players in bracket
    pub fn update_seeding(&mut self, players: &[PlayerId]) -> Result<(), Error> {
        let mut player_group = Players::default();
        for sorted_player in players {
            let players = self.get_players().get_players();
            let player = match players.iter().find(|p| p.get_id() == *sorted_player) {
                Some(p) => p,
                None => return Err(Error::UnknownPlayer(*sorted_player, self.players.clone())),
            };
            player_group.add(player.clone())?;
        }
        let updated_players = seed(&self.seeding_method, player_group)?;
        if !self.players.contains_same_players(&updated_players) {
            return Err(Error::Players(updated_players, self.players.clone()));
        }
        let updated_matches = get_balanced_round_matches_top_seed_favored(&updated_players)?;
        self.players = updated_players;
        self.matches = updated_matches;
        Ok(())
    }

    /// Allow people to report match results
    pub fn accept_match_results(&mut self) {
        self.accept_match_results = true;
    }

    /// Bar new participants from entering bracket
    pub fn bar_from_entering(&mut self) {
        self.barred_from_entering = true;
    }

    /// Return bracket format
    #[must_use]
    pub fn get_format(&self) -> Format {
        self.format
    }

    /// Returns true if bracket is accepting match results
    #[must_use]
    pub fn is_accepting_match_results(&self) -> bool {
        self.accept_match_results
    }

    /// Returns seeding method
    #[must_use]
    pub fn get_seeding_method(&self) -> SeedingMethod {
        self.seeding_method
    }

    /// Returns true if bracket is barring new participants from entering
    #[must_use]
    pub fn bars_from_entering(&self) -> bool {
        self.barred_from_entering
    }

    /// Returns true if matches are automatically validated when players agree on match result
    #[must_use]
    pub fn is_automatically_validating_matches(&self) -> bool {
        self.automatic_match_validation
    }

    /// Returns start time of bracket
    #[must_use]
    pub fn get_start_time(&self) -> DateTime<Utc> {
        self.start_time
    }

    /// Returns matches
    #[must_use]
    pub fn get_matches(&self) -> Vec<Match> {
        self.matches.clone()
    }

    /// Returns updated bracket
    ///
    /// # Errors
    /// Thrown when the same player is added
    pub fn add_new_player(self, player: Player) -> Result<Bracket, Error> {
        let mut players = self.players.clone();
        players.add(player)?;
        let matches = if players.len() < 3 {
            vec![]
        } else {
            get_balanced_round_matches_top_seed_favored(&players)?
        };
        Ok(Self {
            bracket_id: self.bracket_id,
            bracket_name: self.bracket_name,
            players,
            matches,
            format: self.format,
            seeding_method: self.seeding_method,
            start_time: self.start_time,
            accept_match_results: self.accept_match_results,
            automatic_match_validation: self.automatic_match_validation,
            barred_from_entering: self.barred_from_entering,
        })
    }

    /// Report match result
    ///
    /// # Errors
    /// Thrown when given match id does not correspond to any match in the bracket
    pub fn update_match_result(
        &mut self,
        match_id: MatchId,
        result: ReportedResult,
        player_id: PlayerId,
    ) -> Result<(), Error> {
        let m = match self.matches.iter_mut().find(|m| m.get_id() == match_id) {
            Some(m) => m,
            None => return Err(Error::UnknownMatch(match_id)),
        };

        m.update_reported_result(player_id, result)?;
        Ok(())
    }

    /// Validate match result. If final match is validated, then bracket will
    /// stop accepting match result:
    ///
    /// # Errors
    /// Thrown when given match id is unknown or when reported results differ
    pub fn validate_match_result(&mut self, match_id: MatchId) -> Result<(), Error> {
        let (updated_match_id, seed_of_expected_winner, winner) =
            match self.matches.iter_mut().find(|m| m.get_id() == match_id) {
                Some(m) => {
                    let seed_of_expected_winner = m.set_outcome()?;
                    (m.get_id(), seed_of_expected_winner, m.get_winner())
                }
                None => return Err(Error::UnknownMatch(match_id)),
            };
        let last_match = self.matches.last_mut().expect("Last match");
        if last_match.get_id() == match_id {
            self.accept_match_results = false;
            return Ok(());
        }

        let index = self
            .matches
            .iter_mut()
            .position(|m| m.get_id() == updated_match_id)
            .expect("reference to updated match");
        let iter = self.matches.iter_mut();
        let mut iter = iter.skip(index + 1);
        let m = iter
            .find(|m| m.get_seeds().contains(&seed_of_expected_winner))
            .expect("Match to update with player moving in the bracket");
        if m.get_seeds()[0] == seed_of_expected_winner {
            m.players[0] = winner;
        } else {
            m.players[1] = winner;
        }

        Ok(())
    }
}

impl From<Bracket> for Raw {
    fn from(b: Bracket) -> Self {
        Self {
            bracket_id: b.get_id(),
            bracket_name: b.get_name(),
            players: b
                .get_players()
                .get_players()
                .iter()
                .map(Player::get_id)
                .collect(),
            player_names: b
                .get_players()
                .get_players()
                .iter()
                .map(Player::get_name)
                .collect(),
            matches: b.matches.clone(),
            format: b.get_format(),
            seeding_method: b.get_seeding_method(),
            start_time: b.get_start_time(),
            accept_match_results: b.is_accepting_match_results(),
            automatic_match_validation: b.is_automatically_validating_matches(),
            barred_from_entering: b.bars_from_entering(),
        }
    }
}

/// Error while parsing bracket
#[derive(Debug)]
pub enum ParsingBracketError {
    /// Malformed players
    Players(PlayerError),
}

impl From<PlayerError> for ParsingBracketError {
    fn from(e: PlayerError) -> Self {
        Self::Players(e)
    }
}

impl std::fmt::Display for ParsingBracketError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParsingBracketError::Players(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for ParsingBracketError {}

impl TryFrom<Raw> for Bracket {
    type Error = ParsingBracketError;

    fn try_from(br: Raw) -> Result<Self, Self::Error> {
        Ok(Self {
            bracket_id: br.bracket_id,
            bracket_name: br.bracket_name.clone(),
            players: {
                let players: Vec<(&Uuid, &String)> =
                    br.players.iter().zip(br.player_names.iter()).collect();
                Players::try_from(players)?
            },
            matches: br.matches,
            format: br.format,
            seeding_method: br.seeding_method,
            start_time: br.start_time,
            accept_match_results: br.accept_match_results,
            automatic_match_validation: br.automatic_match_validation,
            barred_from_entering: br.barred_from_entering,
        })
    }
}

impl Display for Raw {
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
    brackets: Vec<Raw>,
}

impl Brackets {
    /// Create representation of brackets implementing `std::fmt::Display`
    #[must_use]
    pub fn new(brackets: Vec<Raw>) -> Self {
        Brackets { brackets }
    }

    /// Get brackets
    #[must_use]
    pub fn get_brackets(&self) -> Vec<Raw> {
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

impl Raw {
    /// Create new bracket
    #[must_use]
    pub fn new(
        bracket_name: String,
        format: Format,
        seeding_method: SeedingMethod,
        start_time: DateTime<Utc>,
        automatic_match_validation: bool,
    ) -> Self {
        // TODO add check where registration_start_time < beginning_start_time
        Raw {
            bracket_id: Uuid::new_v4(),
            bracket_name,
            players: vec![],
            player_names: vec![],
            matches: vec![],
            format,
            seeding_method,
            start_time,
            accept_match_results: false,
            automatic_match_validation,
            barred_from_entering: false,
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

    /// Get players ids
    #[must_use]
    pub fn get_player_ids(&self) -> Vec<PlayerId> {
        self.players.clone()
    }

    /// Get players
    #[must_use]
    pub fn get_players(&self) -> Vec<Player> {
        self.players
            .iter()
            .zip(self.player_names.iter())
            .map(|p| Player {
                id: *p.0,
                name: p.1.to_string(),
            })
            .collect()
    }

    /// Seed bracket
    ///
    /// # Errors
    /// Returns an error if necessary arguments for seeding are missing
    pub fn seed(self, players: Option<Players>) -> Result<Raw, Error> {
        let matches: Vec<Match> =
            match self.format {
                Format::SingleElimination => match players {
                    Some(players) => {
                        let players = seed(&self.seeding_method, players)?;
                        get_balanced_round_matches_top_seed_favored(&players)?
                    }
                    None => return Err(Error::MissingArgument(
                        "players".to_string(),
                        "Cannot seed single-elimination bracket without a list of seeded player"
                            .to_string(),
                    )),
                },
            };
        let mut bracket = self;
        bracket.matches = matches;
        Ok(bracket)
    }

    /// Get matches
    #[must_use]
    pub fn get_matches(&self) -> Vec<Match> {
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

    /// Accept match results
    pub fn accept_match_results(&mut self) {
        self.accept_match_results = true;
    }

    /// Returns true if bracket accepts match results
    #[must_use]
    pub fn get_accept_match_results(&self) -> bool {
        self.accept_match_results
    }

    /// Returns true if match is validated automatically when both players agree
    #[must_use]
    pub fn get_automatic_match_validation(&self) -> bool {
        self.automatic_match_validation
    }

    /// Bar new participants from entering bracket
    pub fn bar_from_entering(&mut self) {
        self.barred_from_entering = true;
    }

    /// Returns true if new participants are barred from entering bracket
    #[must_use]
    pub fn is_barred_from_entering(&self) -> bool {
        self.barred_from_entering
    }
}

impl From<SeedingError> for Error {
    fn from(e: SeedingError) -> Self {
        Self::Seeding(e)
    }
}

impl From<PlayerError> for Error {
    fn from(pe: PlayerError) -> Self {
        Self::PlayerUpdate(pe)
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
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct GET {
    /// Identifier of bracket
    pub bracket_id: Id,
    /// Name of this bracket
    pub bracket_name: String,
    /// Players in this bracket
    pub players: Vec<Player>,
    /// Matches for this bracket
    pub matches: Vec<MatchGET>,
    /// Bracket format
    pub format: String,
    /// Seeding method used for this bracket
    pub seeding_method: String,
    /// Advertised start time
    pub start_time: String,
    /// Accept match results
    pub accept_match_results: bool,
    /// Automatically validate match results if both players agree
    pub automatic_match_validation: bool,
    /// Bar new participants from entering
    pub barred_from_entering: bool,
}

impl GET {
    /// Form values to be sent to the API to create a bracket
    #[must_use]
    pub fn new(bracket: &Raw) -> Self {
        GET {
            bracket_id: bracket.get_id(),
            bracket_name: bracket.get_bracket_name(),
            players: bracket.get_players(),
            format: bracket.get_format().to_string(),
            seeding_method: bracket.get_seeding_method().to_string(),
            matches: bracket
                .matches
                .clone()
                .into_iter()
                .map(std::convert::Into::into)
                .collect::<Vec<MatchGET>>(),
            start_time: bracket.start_time.to_string(),
            accept_match_results: bracket.accept_match_results,
            automatic_match_validation: bracket.automatic_match_validation,
            barred_from_entering: bracket.barred_from_entering,
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

impl TryFrom<GET> for Raw {
    type Error = ParsingError;

    fn try_from(b: GET) -> Result<Self, Self::Error> {
        Ok(Self {
            bracket_id: b.bracket_id,
            bracket_name: b.bracket_name,
            players: b.players.iter().map(Player::get_id).collect(),
            player_names: b.players.iter().map(Player::get_name).collect(),
            matches: b
                .matches
                .into_iter()
                .map(Match::try_from)
                .collect::<Result<Vec<Match>, _>>()?,
            format: b.format.parse::<Format>()?,
            seeding_method: b.seeding_method.parse::<SeedingMethod>()?,
            start_time: b.start_time.parse::<DateTime<Utc>>()?,
            accept_match_results: b.accept_match_results,
            automatic_match_validation: b.automatic_match_validation,
            barred_from_entering: b.barred_from_entering,
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
    /// Automatically validate match if both players agree
    pub automatic_match_validation: bool,
}

impl From<Raw> for GET {
    fn from(b: Raw) -> Self {
        GET::new(&b)
    }
}

impl From<chrono::ParseError> for ParsingError {
    fn from(e: chrono::ParseError) -> Self {
        Self::Time(e)
    }
}

/// Report match result
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct StartBracketPOST {
    /// Discussion channel id of service
    pub channel_internal_id: String,
    /// Service used to make call
    pub service_type_id: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::matches::Opponent;

    #[test]
    fn updating_seeding_changes_matches_of_3_man_bracket() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id];
        let player_names = vec!["p1".to_string(), "p2".to_string(), "p3".to_string()];
        let players = Players::from_raw_id(
            player_ids
                .clone()
                .iter()
                .zip(player_names.clone().iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let mut bracket: Bracket = Raw {
            bracket_id: BracketId::new_v4(),
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        bracket
            .update_seeding(&[p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> =
            bracket.get_matches().iter().map(|m| m.get_id()).collect();
        match_ids.reverse();
        let p1 = Opponent::Player(p1_id);
        let p2 = Opponent::Player(p2_id);
        let p3 = Opponent::Player(p3_id);
        assert_eq!(
            bracket.get_matches(),
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [p2, p1],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [p3, Opponent::Unknown],
                    [1, 2],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                )
                .expect("match")
            ]
        );
    }

    #[test]
    fn updating_seeding_changes_matches_of_5_man_bracket() {
        let p1_id = PlayerId::new_v4();
        let p2_id = PlayerId::new_v4();
        let p3_id = PlayerId::new_v4();
        let p4_id = PlayerId::new_v4();
        let p5_id = PlayerId::new_v4();
        let player_ids = vec![p1_id, p2_id, p3_id, p4_id, p5_id];
        let player_names = vec![
            "p1".to_string(),
            "p2".to_string(),
            "p3".to_string(),
            "p4".to_string(),
            "p5".to_string(),
        ];
        let players = Players::from_raw_id(
            player_ids
                .clone()
                .iter()
                .zip(player_names.clone().iter())
                .map(|p| (p.0.to_string(), p.1.clone()))
                .collect(),
        )
        .expect("players");
        let matches = get_balanced_round_matches_top_seed_favored(&players).expect("matches");
        let mut bracket: Bracket = Raw {
            bracket_id: BracketId::new_v4(),
            bracket_name: "bracket".to_string(),
            players: player_ids,
            player_names,
            matches,
            format: Format::SingleElimination,
            seeding_method: SeedingMethod::Strict,
            start_time: Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            accept_match_results: false,
            automatic_match_validation: false,
            barred_from_entering: true,
        }
        .try_into()
        .expect("bracket");
        bracket
            .update_seeding(&[p4_id, p5_id, p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> =
            bracket.get_matches().iter().map(|m| m.get_id()).collect();
        match_ids.reverse();
        let p1 = Opponent::Player(p1_id);
        let p2 = Opponent::Player(p2_id);
        let p3 = Opponent::Player(p3_id);
        let p4 = Opponent::Player(p4_id);
        let p5 = Opponent::Player(p5_id);
        assert_eq!(
            bracket.get_matches(),
            vec![
                Match::from(
                    match_ids.pop().expect("match id"),
                    [p2, p1],
                    [4, 5],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [p4, Opponent::Unknown],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [p5, p3],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                )
                .expect("match"),
                Match::from(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [1, 2],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                )
                .expect("match"),
            ]
        );
    }
}
