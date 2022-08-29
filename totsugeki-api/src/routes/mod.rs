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
        let msg = e.to_string();
        match e {
            Error::PoisonedReadLock(_)
            | Error::PoisonedWriteLock(_)
            | Error::Code(_)
            | Error::Unknown(_)
            | Error::NextMatchNotFound
            | Error::Seeding(_) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::UnregisteredBracket(_)
            | Error::UnregisteredDiscussionChannel(_, _)
            | Error::NoActiveBracketInDiscussionChannel
            | Error::NoNextMatch
            | Error::PlayerNotFound
            | Error::NoOpponent
            | Error::Match(_)
            | Error::EliminatedFromBracket
            | Error::OrganiserNotFound(_, _) => pError::from_string(msg, StatusCode::NOT_FOUND),
            Error::BracketInactive(_, _) => pError::from_string(msg, StatusCode::UNAUTHORIZED),
            Error::Denied(_) | Error::UpdateBracket(_) => {
                pError::from_string(msg, StatusCode::FORBIDDEN)
            }
            Error::Parsing(_) => pError::from_string(msg, StatusCode::BAD_REQUEST),
        }
    }
}
