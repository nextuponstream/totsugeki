//! Bracket domain

use crate::{
    bracket::Id as BracketId,
    format::{Format, ParsingError as FormatParsingError},
    matches::{
        Error as MatchError, Id as MatchId, Match, MatchGET, MatchParsingError, ReportedResult,
    },
    opponent::Opponent,
    organiser::Id as OrganiserId,
    player::{Error as PlayerError, Id as PlayerId, Player, Players},
    seeding::{
        get_balanced_round_matches_top_seed_favored, seed, Error as SeedingError,
        Method as SeedingMethod, ParsingError as SeedingParsingError,
    },
    DiscussionChannelId,
};
use chrono::prelude::*;
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fmt::Display,
};
use thiserror::Error;
use uuid::Uuid;

/// Updating bracket cannot be performed or searched information does not exist
#[derive(Error, Debug)]
pub enum Error {
    /// Error while seeding a bracket
    #[error("{0}")]
    Seeding(#[from] SeedingError),
    /// Error while updating players of bracket
    #[error("{0}")]
    PlayerUpdate(#[from] PlayerError),
    /// Match referred does not exist for this bracket
    #[error("Match with id \"{0}\" does not exists in bracket")]
    UnknownMatch(MatchId),
    /// Match cannot be updated
    #[error("{0}")]
    Match(#[from] MatchError),
    /// Unknown player provided for seeding
    #[error("Unknown player \"{0}\" cannot be used for seeding. Use the following players: {1} of bracket {2}")]
    UnknownPlayer(PlayerId, Players, BracketId),
    /// Provided players for seeding do not match players in bracket
    #[error("Provided players cannot be used: {0}\nUse the following players: {1}")]
    Players(Players, Players),
    /// Cannot add player when they are barred from entering
    #[error("Bracket \"{1}\" does not accept new participants")]
    BarredFromEntering(PlayerId, BracketId),
    /// Bracket does not accept result at the moment
    ///
    /// This happens when bracket has not started yet or has ended
    #[error("Bracket \"{1}\" does not accept reported results at the moment")]
    AcceptResults(PlayerId, BracketId),
    /// Player reported a result but there is no match for him to play
    #[error("There is no match to update in bracket \"{1}\"")]
    NoMatchToPlay(PlayerId, BracketId),
    /// No matches where generated for this bracket
    #[error("No matches were generated yet for bracket {0}")]
    NoGeneratedMatches(BracketId),
    /// There is no match to play because player won the bracket
    #[error("There is no match for you to play because you won the bracket {1}")]
    NoNextMatch(PlayerId, BracketId),
    /// There is no match to play for player querying because he was eliminated
    /// from the bracket
    #[error("There is no match for you to play because you were eliminated from bracket {1}")]
    EliminatedFromBracket(PlayerId, BracketId),
    /// Only player in bracket can query for their next opponent
    #[error(
        "You do not have next opponent because you are not a participant of this bracket (\"{1}\")"
    )]
    PlayerIsNotParticipant(PlayerId, Id),
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
#[derive(Clone, Debug)]
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
    /// When set to `true`, accept match results
    accept_match_results: bool,
    /// Matches are automatically validated if both players agree on result
    automatic_match_validation: bool,
    /// When set to `true`, bars new participants from entering bracket
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
    pub fn update_seeding(self, players: &[PlayerId]) -> Result<Self, Error> {
        let mut player_group = Players::default();
        for sorted_player in players {
            let players = self.get_players().get_players_list();
            let player = match players.iter().find(|p| p.get_id() == *sorted_player) {
                Some(p) => p,
                None => {
                    return Err(Error::UnknownPlayer(
                        *sorted_player,
                        self.players.clone(),
                        self.bracket_id,
                    ))
                }
            };
            player_group.add(player.clone())?;
        }
        let updated_players = seed(&self.seeding_method, player_group)?;
        if !self.players.contains_same_players(&updated_players) {
            return Err(Error::Players(updated_players, self.players));
        }
        let updated_matches = get_balanced_round_matches_top_seed_favored(&updated_players)?;
        Ok(Self {
            players: updated_players,
            matches: updated_matches,
            ..self
        })
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

    /// Returns seeding method
    #[must_use]
    pub fn get_seeding_method(&self) -> SeedingMethod {
        self.seeding_method
    }

    /// Returns true if bracket is barring new participants from entering
    #[must_use]
    fn bars_from_entering(&self) -> bool {
        self.barred_from_entering
    }

    /// Returns start time of bracket
    #[must_use]
    fn get_start_time(&self) -> DateTime<Utc> {
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

    /// Report match result and returns updated bracket
    ///
    /// # Errors
    /// Thrown when given match id does not correspond to any match in the bracket
    fn update_match_result(
        self,
        match_id: MatchId,
        result: ReportedResult,
        player_id: PlayerId,
    ) -> Result<Bracket, Error> {
        let m = match self.matches.iter().find(|m| m.get_id() == match_id) {
            Some(m) => m,
            None => return Err(Error::UnknownMatch(match_id)),
        };

        let updated_match = m.update_reported_result(player_id, result)?;
        let updated_matches = self
            .matches
            .clone()
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match
                } else {
                    *m
                }
            })
            .collect();
        let mut updated_bracket = self;
        updated_bracket.matches = updated_matches;
        Ok(updated_bracket)
    }

    /// Validate match result and return updated bracket. Winner moves forward
    /// in bracket. If final match is validated, then bracket will stop
    /// accepting match result.
    ///
    /// # Errors
    /// Thrown when given match id is unknown or when reported results differ
    pub fn validate_match_result(self, match_id: MatchId) -> Result<Self, Error> {
        // declare winner if there is one
        let mut updated_bracket = self.clone();
        let (updated_match, seed_of_expected_winner, winner_id) = match self
            .matches
            .clone()
            .iter_mut()
            .find(|m| m.get_id() == match_id)
        {
            Some(m) => m.update_outcome()?,
            None => return Err(Error::UnknownMatch(match_id)),
        };
        let mut updated_matches = updated_bracket.matches.clone();
        updated_matches = updated_matches
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match
                } else {
                    *m
                }
            })
            .collect();
        updated_bracket.matches = updated_matches;

        let last_match = updated_bracket.matches.last().expect("last match");
        if last_match.get_id() == match_id {
            updated_bracket.accept_match_results = false;
            return Ok(updated_bracket);
        }

        // winner moves forward in bracket
        let index = self
            .matches
            .iter()
            .position(|m| m.get_id() == updated_match.get_id())
            .expect("reference to updated match");
        let mut updated_matches = updated_bracket.matches.clone();
        let iter = updated_matches.iter_mut();
        let mut iter = iter.skip(index + 1);
        let m = iter
            .find(|m| m.get_seeds().contains(&seed_of_expected_winner))
            .expect("match where winner plays next");
        let updated_match = m.set_player(winner_id, m.get_seeds()[0] == seed_of_expected_winner);
        updated_bracket.matches = updated_bracket
            .matches
            .iter()
            .map(|m| {
                if m.get_id() == updated_match.get_id() {
                    updated_match
                } else {
                    *m
                }
            })
            .collect();
        Ok(updated_bracket)
    }

    /// Start bracket: bar people from entering and accept match results
    pub fn start(&mut self) {
        self.bar_from_entering();
        self.accept_match_results = true;
    }

    /// Let `player` join participants and returns an updated version of the bracket
    ///
    /// # Errors
    /// Thrown when bracket has already started
    pub fn join(self, player: Player) -> Result<Bracket, Error> {
        if self.bars_from_entering() {
            return Err(Error::BarredFromEntering(player.get_id(), self.get_id()));
        }
        let updated_bracket = self.add_new_player(player)?;
        Ok(updated_bracket)
    }

    /// Report result for a match in this bracket. Returns updated bracket and
    /// affected match id
    ///
    /// # Errors
    /// Thrown when result cannot be parsed
    pub fn report_result(
        self,
        player_id: PlayerId,
        result: ReportedResult,
    ) -> Result<(Bracket, MatchId), Error> {
        if !self.accept_match_results {
            return Err(Error::AcceptResults(player_id, self.bracket_id));
        }
        let player = Opponent::Player(player_id);
        let match_to_update = self.matches.iter().find(|m| {
            (m.get_players()[0] == player || m.get_players()[1] == player)
                && m.get_winner() == Opponent::Unknown
        });
        let bracket_id = self.get_id();
        match match_to_update {
            Some(m) => {
                let affected_match_id = m.get_id();
                let updated_bracket =
                    self.update_match_result(affected_match_id, result, player_id)?;
                Ok((updated_bracket, affected_match_id))
            }
            None => Err(Error::NoMatchToPlay(player_id, bracket_id)),
        }
    }

    /// Return next opponent for `player_id`, relevant match and player name
    ///
    /// # Errors
    /// Thrown when matches have yet to be generated or player has won/been
    /// eliminated
    pub fn next_opponent(&self, player_id: PlayerId) -> Result<(Opponent, MatchId, String), Error> {
        if !self
            .players
            .clone()
            .get_players_list()
            .iter()
            .map(Player::get_id)
            .any(|id| id == player_id)
        {
            return Err(Error::PlayerIsNotParticipant(player_id, self.bracket_id));
        }
        if self.matches.is_empty() {
            return Err(Error::NoGeneratedMatches(self.bracket_id));
        }

        let player = Opponent::Player(player_id);
        let next_match = self.matches.iter().find(|m| {
            (m.get_players()[0] == player || m.get_players()[1] == player)
                && m.get_winner() == Opponent::Unknown
        });
        let relevant_match = match next_match {
            Some(m) => m,
            None => {
                let last_match = self.matches.iter().last().expect("last match");
                if let Opponent::Player(id) = last_match.get_winner() {
                    if id == player_id {
                        return Err(Error::NoNextMatch(player_id, self.bracket_id));
                    }
                }
                return Err(Error::EliminatedFromBracket(player_id, self.bracket_id));
            }
        };

        let opponent = if relevant_match.get_players()[0] == player {
            relevant_match.get_players()[1]
        } else {
            relevant_match.get_players()[0]
        };
        let player_name = match opponent {
            Opponent::Player(opponent_id) => self
                .players
                .clone()
                .get_players_list()
                .iter()
                .find(|p| p.id == opponent_id)
                .map_or_else(|| Opponent::Unknown.to_string(), Player::get_name),
            Opponent::Unknown => Opponent::Unknown.to_string(),
        };

        Ok((opponent, relevant_match.get_id(), player_name))
    }
}

impl From<Bracket> for Raw {
    fn from(b: Bracket) -> Self {
        Self {
            bracket_id: b.get_id(),
            bracket_name: b.get_name(),
            players: b
                .get_players()
                .get_players_list()
                .iter()
                .map(Player::get_id)
                .collect(),
            player_names: b
                .get_players()
                .get_players_list()
                .iter()
                .map(Player::get_name)
                .collect(),
            matches: b.matches.clone(),
            format: b.get_format(),
            seeding_method: b.get_seeding_method(),
            start_time: b.get_start_time(),
            accept_match_results: b.accept_match_results,
            automatic_match_validation: b.automatic_match_validation,
            barred_from_entering: b.bars_from_entering(),
        }
    }
}

impl TryFrom<Raw> for Bracket {
    type Error = ParsingError;

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

/// Collection of bracket (raw data)
#[derive(Default)]
pub struct RawBrackets {
    /// A collection of brackets
    brackets: Vec<Raw>,
}

impl RawBrackets {
    /// Create representation of brackets implementing `std::fmt::Display`
    #[must_use]
    pub fn new(brackets: Vec<Raw>) -> Self {
        RawBrackets { brackets }
    }

    /// Get brackets
    #[must_use]
    pub fn get_brackets(&self) -> Vec<Raw> {
        self.brackets.clone()
    }
}

impl Display for RawBrackets {
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

    /// Get players
    #[must_use]
    pub fn get_players_list(&self) -> Vec<Player> {
        self.players
            .iter()
            .zip(self.player_names.iter())
            .map(|p| Player {
                id: *p.0,
                name: p.1.to_string(),
            })
            .collect()
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

/// Bracket GET response
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
#[cfg_attr(feature = "poem-openapi", oai(rename = "BracketGET"))]
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
            bracket_id: bracket.bracket_id,
            bracket_name: bracket.bracket_name.clone(),
            players: bracket.get_players_list(),
            format: bracket.format.to_string(),
            seeding_method: bracket.seeding_method.to_string(),
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
#[derive(Error, Debug)]
pub enum ParsingError {
    /// Could not parse bracket format
    #[error("{0}")]
    Format(#[from] FormatParsingError),
    /// Could not parse seeding method
    #[error("{0}")]
    Seeding(#[from] SeedingParsingError),
    /// Could not parse match
    #[error("{0}")]
    Match(#[from] MatchParsingError),
    /// Could not parse time
    #[error("{0}")]
    Time(#[from] chrono::ParseError),
    /// Could not parse players
    #[error("{0}")]
    Players(#[from] PlayerError),
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

impl From<Raw> for GET {
    fn from(b: Raw) -> Self {
        GET::new(&b)
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

impl std::fmt::Display for Bracket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.bracket_name, self.bracket_id)
    }
}

/// Parameters to create a bracket
pub struct CreateRequest<'b> {
    /// Automatically validate match if both players agree
    pub automatic_match_validation: bool,
    /// requested bracket format
    pub bracket_format: &'b str,
    /// requested bracket name
    pub bracket_name: &'b str,
    /// Id of internal channel
    pub internal_channel_id: &'b str,
    /// Organiser id of requested bracket while using service
    pub organiser_internal_id: &'b str,
    /// Organiser name of requested bracket
    pub organiser_name: &'b str,
    /// seeding method of requested bracket
    pub seeding_method: &'b str,
    /// Type of service used to make request
    pub service_type_id: &'b str,
    /// Advertised start time
    pub start_time: &'b str,
}

impl TryFrom<CreateRequest<'_>> for Bracket {
    type Error = ParsingError;

    fn try_from(br: CreateRequest) -> Result<Self, Self::Error> {
        Ok(Bracket::new(
            br.bracket_name,
            br.bracket_format.parse()?,
            br.seeding_method.parse()?,
            br.start_time.parse()?,
            br.automatic_match_validation,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::opponent::Opponent;

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
        let bracket: Bracket = Raw {
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
        let updated_bracket = bracket
            .update_seeding(&[p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(|m| m.get_id())
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(p1_id);
        let p2 = Opponent::Player(p2_id);
        let p3 = Opponent::Player(p3_id);
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p2, p1],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p3, Opponent::Unknown],
                    [1, 2],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
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
        let bracket: Bracket = Raw {
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
        let updated_bracket = bracket
            .update_seeding(&[p4_id, p5_id, p3_id, p2_id, p1_id])
            .expect("seeding update");
        let mut match_ids: Vec<MatchId> = updated_bracket
            .get_matches()
            .iter()
            .map(|m| m.get_id())
            .collect();
        match_ids.reverse();
        let p1 = Opponent::Player(p1_id);
        let p2 = Opponent::Player(p2_id);
        let p3 = Opponent::Player(p3_id);
        let p4 = Opponent::Player(p4_id);
        let p5 = Opponent::Player(p5_id);
        assert_eq!(
            updated_bracket.get_matches(),
            vec![
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p2, p1],
                    [4, 5],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p4, Opponent::Unknown],
                    [1, 4],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [p5, p3],
                    [2, 3],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
                Match::try_from(MatchGET::new(
                    match_ids.pop().expect("match id"),
                    [Opponent::Unknown, Opponent::Unknown],
                    [1, 2],
                    Opponent::Unknown,
                    Opponent::Unknown,
                    [(0, 0), (0, 0)]
                ))
                .expect("match"),
            ]
        );
    }

    #[test]
    fn new_participants_can_join_bracket() {
        let mut bracket = Bracket::new(
            "name",
            Format::default(),
            SeedingMethod::default(),
            Utc.ymd(2000, 1, 1).and_hms(0, 0, 0),
            false,
        );
        for i in 0..10 {
            bracket = bracket
                .join(Player::new(format!("player{i}")))
                .expect("updated_bracket");
        }
    }

    #[test]
    fn starting_bracket_will_deny_new_participants_from_entering() {
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
        bracket.start();
        let bracket_id = bracket.get_id();

        let player = Player::new("New player".to_string());
        let player_id = player.get_id();
        let err = bracket
            .join(player)
            .expect_err("JoinAfterBracketHasStarted");
        match err {
            Error::BarredFromEntering(id, b_id) => {
                assert_eq!(id, player_id);
                assert_eq!(b_id, bracket_id);
            }
            _ => assert!(false, "expected BarredFromEntering error, got: {}", err),
        };
    }
}
