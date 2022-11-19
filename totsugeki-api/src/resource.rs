//! Unknown resource

use crate::Service;
use thiserror::Error;
use totsugeki::{
    bracket::{matches::Error as ProgressionError, Error as BracketError},
    DiscussionChannelId,
};

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
            BracketError::UnknownPlayer(_, _, _)
            | BracketError::Progression(
                _,
                ProgressionError::Disqualified(_)
                | ProgressionError::NoGeneratedMatches
                | ProgressionError::Eliminated(_)
                | ProgressionError::NoNextMatch(_)
                | ProgressionError::NoMatchToPlay(_),
            ) => Self::UnknownBracket(e),

            BracketError::PlayerUpdate(_)
            | BracketError::BarredFromEntering(_, _)
            | BracketError::Started(_, _)
            | BracketError::NotStarted(_, _)
            | BracketError::Progression(_, _)
            | BracketError::Seeding(_) => Self::ForbiddenBracketUpdate(e),
        }
    }
}
