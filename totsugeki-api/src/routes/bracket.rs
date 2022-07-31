//! bracket routes
use crate::bracket::{POSTResult, POST};
use crate::log_error;
use crate::persistence::Error;
use crate::ApiKeyServiceAuthorization;
use crate::GETResponse;
use crate::Service;
use crate::SharedDb;
use poem::http::StatusCode;
use poem::Error as pError;
use poem::Result;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use totsugeki::bracket::Bracket;

/// Bracket Api
pub struct Api;

#[OpenApi]
impl Api {
    /// Create a new active bracket in issued discussion channel
    #[oai(path = "/bracket", method = "post")]
    async fn create_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        bracket_request: Json<POST>,
    ) -> Result<Json<POSTResult>> {
        match create_new_active_bracket(
            &db,
            bracket_request.bracket_name.as_str(),
            bracket_request.organiser_name.as_str(),
            bracket_request.organiser_internal_id.clone(),
            bracket_request.channel_internal_id.clone(),
            bracket_request.service_type_id.as_str(),
        ) {
            Ok(r) => Ok(Json(r)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// List registered brackets
    #[oai(path = "/bracket/:offset", method = "get")]
    async fn list_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<GETResponse>>> {
        match list_brackets(&db, offset.0) {
            Ok(brackets) => {
                let mut b_api_vec = vec![];
                for b in brackets {
                    b_api_vec.push(b.try_into()?);
                }
                Ok(Json(b_api_vec))
            }
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Matches exactly bracket name
    #[oai(path = "/bracket/:bracket_name/:offset", method = "get")]
    async fn find_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        bracket_name: Path<String>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<GETResponse>>> {
        match find_bracket(&db, bracket_name.0.as_str(), offset.0) {
            Ok(brackets) => {
                let mut b_api_vec = vec![];
                for b in brackets {
                    let b_api: GETResponse = b.try_into()?;
                    b_api_vec.push(b_api);
                }
                Ok(Json(b_api_vec))
            }
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }
}

impl<'a> From<Error<'a>> for pError {
    fn from(e: Error<'a>) -> Self {
        match e {
            Error::PoisonedReadLock(_e) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::PoisonedWriteLock(_e) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::Code(_msg) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
            Error::Denied(msg) => pError::from_string(msg, StatusCode::FORBIDDEN),
            Error::Parsing(msg) => pError::from_string(msg, StatusCode::BAD_REQUEST),
            Error::Unknown(_msg) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

/// Database call to create new active bracket from issued discussion channel
fn create_new_active_bracket<'a, 'b, 'c>(
    db: &'a SharedDb,
    bracket_name: &'b str,
    organiser_name: &'b str,
    organiser_id: String,
    internal_channel_id: String,
    service_type_id: &str,
) -> Result<POSTResult, Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    let service_type_id = match service_type_id.parse::<Service>() {
        Ok(v) => v,
        Err(e) => {
            return Err(Error::Parsing(format!("{e:?}")));
        }
    };
    let result = db.create_bracket(
        bracket_name,
        organiser_name,
        organiser_id,
        internal_channel_id,
        service_type_id,
    )?;
    let result = result.into();
    Ok(result)
}

/// Call to database to list registered bracket
fn list_brackets<'a, 'b>(db: &'a SharedDb, offset: i64) -> Result<Vec<Bracket>, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.list_brackets(offset)
}

/// Call to database to find bracket named `bracket_name`
fn find_bracket<'a, 'b, 'c>(
    db: &'a SharedDb,
    bracket_name: &'b str,
    offset: i64,
) -> Result<Vec<Bracket>, Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    db.find_brackets(bracket_name, offset)
}
