//! Parse user input and return error so input can be corrected by user

use crate::{ParseServiceInternalIdError, Service};
use serenity::model::{
    id::{ChannelId, GuildId, UserId},
    misc::{ChannelIdParseError, UserIdParseError},
};
use std::num::ParseIntError;
use thiserror::Error;
use totsugeki::{
    bracket::{Bracket, CreateRequest, ParsingError as BracketParsingError},
    matches::{MatchResultParsingError, ReportedResult},
    player::Id as PlayerIdentifier,
};

/// User input cannot be parsed. User can adapt his input
// NOTE: while you could only parse discord id and feed it into a XX::from()
// method, then you lose where the error happens: is it the channel id? user
// id? ... While it is not exactly practical to write an enum variant for each
// cases, it's better if the error message is exact rather than approximative:
// the user can adapt his input accordingly
#[derive(Error, Debug)]
pub enum Error {
    /// Error with provided bracket
    #[error("bracket = {0}")]
    Bracket(#[from] BracketParsingError),
    /// Error with match result
    #[error("match result = {0}")]
    MatchResult(#[from] MatchResultParsingError),
    /// Error with player id
    #[error("player id = {0}")]
    PlayerId(#[from] uuid::Error),
    /// Error with service
    #[error("service = {0}")]
    Service(#[from] ParseServiceInternalIdError),
    /// Error with discord user id
    #[error("discord user id = {0}")]
    DiscordUserId(#[from] UserIdParseError),
    /// Error with discord channel id
    #[error("discord channel id = {0}")]
    DiscordChannelId(#[from] ChannelIdParseError),
    /// Error with guild id
    #[error("discord guild id = {0}")]
    DiscordGuildId(#[from] ParseIntError),
}

/// Parse bracket from user input
pub fn parse_bracket(r: CreateRequest) -> Result<Bracket, Error> {
    Ok(Bracket::try_from(r)?)
}

/// Parse match result from user input
pub fn parse_match_result(match_result: &str) -> Result<ReportedResult, Error> {
    Ok(match_result.parse()?)
}

/// Parse list of player ids from user input
pub fn parse_players(players: &[String]) -> Result<Vec<PlayerIdentifier>, Error> {
    Ok(players
        .iter()
        .map(|p| PlayerIdentifier::parse_str(p))
        .collect::<Result<Vec<_>, _>>()?)
}

/// Parse service from user input
pub fn parse_service(service: &str) -> Result<Service, Error> {
    Ok(service.parse::<Service>()?)
}

/// Parse discord user id
pub fn parse_discord_user_id(id: &str) -> Result<UserId, Error> {
    Ok(id.parse::<UserId>()?)
}

/// Parse discord channel id
pub fn parse_discord_channel_id(id: &str) -> Result<ChannelId, Error> {
    Ok(id.parse::<ChannelId>()?)
}

/// Parse discord guild id
pub fn parse_discord_guild_id(id: &str) -> Result<GuildId, Error> {
    let id = id.parse::<u64>().map_err(Error::DiscordGuildId)?;
    // NOTE: no parse method at time of writting
    Ok(GuildId::from(id))
}
