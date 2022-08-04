//! routes for tournament server

use crate::persistence::Error;
use poem::http::StatusCode;
use poem::Error as pError;

pub mod bracket;
pub mod health_check;
pub mod join;
pub mod organiser;
pub mod service;
pub mod test_utils;

impl<'a> From<Error<'a>> for pError {
    fn from(e: Error<'a>) -> Self {
        match e {
            Error::PoisonedReadLock(_e) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::PoisonedWriteLock(_e) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::Code(_msg) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::Denied(msg) => pError::from_string(msg, StatusCode::FORBIDDEN),
            Error::Parsing(msg) => pError::from_string(msg, StatusCode::BAD_REQUEST),
            Error::Unknown(_msg) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::BracketNotFound(_) => pError::from_status(StatusCode::NOT_FOUND),
            Error::DiscussionChannelNotFound => pError::from_string(
                Error::DiscussionChannelNotFound.to_string(),
                StatusCode::NOT_FOUND,
            ),
            Error::NoActiveBracketInDiscussionChannel => pError::from_string(
                Error::NoActiveBracketInDiscussionChannel.to_string(),
                StatusCode::NOT_FOUND,
            ),
            Error::PlayerNotFound => {
                pError::from_string(Error::PlayerNotFound.to_string(), StatusCode::NOT_FOUND)
            }
            Error::NextMatchNotFound => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::NoNextMatch => {
                pError::from_string(Error::NoNextMatch.to_string(), StatusCode::NOT_FOUND)
            }
            Error::Seeding(_e) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::NoOpponent => {
                pError::from_string(Error::NoOpponent.to_string(), StatusCode::NOT_FOUND)
            }
            Error::Match(e) => pError::from_string(e.to_string(), StatusCode::NOT_FOUND),
            Error::EliminatedFromBracket => pError::from_string(
                Error::EliminatedFromBracket.to_string(),
                StatusCode::NOT_FOUND,
            ),
        }
    }
}
