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
            Error::ParseUserInput(_) => pError::from_string(msg, StatusCode::BAD_REQUEST),
            Error::NotFound(_) => pError::from_string(msg, StatusCode::NOT_FOUND),
            Error::Forbidden(_) => pError::from_string(msg, StatusCode::FORBIDDEN),
            Error::Critical(_) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
