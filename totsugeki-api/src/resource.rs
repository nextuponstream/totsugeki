//! Unknown resource

use crate::Service;
use thiserror::Error;
use totsugeki::{bracket::Error as BracketError, DiscussionChannelId};

/// Ressource is unknown or desired update is forbidden
#[derive(Error, Debug)]
pub enum Error {
    /// Unknown ressource
    #[error("id = {0}")]
    UnknownResource(uuid::Uuid),

    /// Unknown ressource while modifying bracket
    #[error("{0}")]
    UnknownBracket(BracketError),
    /// Forbidden bracket update
    #[error("{0}")]
    ForbiddenBracketUpdate(BracketError),

    /// Unknown discussion channel
    #[error(
        "Unknown discussion channel for service = {0}. Could not map channel id to known channel."
    )]
    UnknownDiscussionChannel(Service),

    /// Unknown player
    #[error("Player is unknown")]
    UnknownPlayer,

    /// Unknown active bracket for discussion channel
    #[error("There is no active bracket in discussion channel ({0})")]
    UnknownActiveBracketForDiscussionChannel(DiscussionChannelId),
}

impl From<BracketError> for Error {
    fn from(e: BracketError) -> Self {
        match e {
            BracketError::Match(_)
            | BracketError::PlayerUpdate(_)
            | BracketError::BarredFromEntering(_, _)
            | BracketError::AcceptResults(_, _)
            | BracketError::Started(_)
            | BracketError::Seeding(_) => Self::ForbiddenBracketUpdate(e),

            BracketError::PlayerIsNotParticipant(_, _)
            | BracketError::EliminatedFromBracket(_, _)
            | BracketError::NoNextMatch(_, _)
            | BracketError::NoGeneratedMatches(_)
            | BracketError::NoMatchToPlay(_, _)
            | BracketError::UnknownPlayer(_, _, _)
            | BracketError::UnknownMatch(_) => Self::UnknownBracket(e),
        }
    }
}
