//! bracket routes
use crate::bracket::{BracketPOST, BracketPOSTResult};
use crate::log_error;
use crate::persistence::Error;
use crate::ApiKeyServiceAuthorization;
use crate::BracketGETResponse;
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
pub struct BracketApi;

#[OpenApi]
impl BracketApi {
    #[oai(path = "/bracket", method = "post")]
    async fn create_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        bracket_request: Json<BracketPOST>,
    ) -> Result<Json<BracketPOSTResult>> {
        match insert_bracket(
            &db,
            bracket_request.bracket_name.as_str(),
            bracket_request.organiser_name.as_str(),
            bracket_request.organiser_internal_id.clone(),
            bracket_request.channel_internal_id.clone(),
            bracket_request.service_type_id.clone(),
        ) {
            Ok(r) => Ok(Json(r)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    #[oai(path = "/bracket/:offset", method = "get")]
    async fn list_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<BracketGETResponse>>> {
        match read_bracket(&db, offset.0) {
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
    ) -> Result<Json<Vec<BracketGETResponse>>> {
        match find_bracket(&db, bracket_name.0.as_str(), offset.0) {
            Ok(brackets) => {
                let mut b_api_vec = vec![];
                for b in brackets {
                    let b_api: BracketGETResponse = b.try_into()?;
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

fn insert_bracket<'a, 'b, 'c>(
    db: &'a SharedDb,
    bracket_name: &'b str,
    organiser_name: &'b str,
    organiser_id: String,
    internal_channel_id: String,
    service_type_id: String,
) -> Result<BracketPOSTResult, Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    let service_type_id = match service_type_id.as_str().parse::<Service>() {
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

fn read_bracket<'a, 'b>(db: &'a SharedDb, offset: i64) -> Result<Vec<Bracket>, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.list_brackets(offset)
}

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
