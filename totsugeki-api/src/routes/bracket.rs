//! bracket routes
use crate::persistence::Error;
use crate::BracketGET;
use crate::BracketPOST;
use crate::MyApiKeyAuthorization;
use crate::SharedDb;
use log::error;
use poem::http::StatusCode;
use poem::Error as pError;
use poem::Result;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use totsugeki::Bracket;

/// Bracket Api
pub struct BracketApi;

#[OpenApi]
impl BracketApi {
    #[oai(path = "/bracket", method = "post")]
    async fn create_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: MyApiKeyAuthorization,
        bracket_request: Json<BracketPOST>,
    ) -> Result<()> {
        match insert_bracket(&db, bracket_request.bracket_name.as_str()) {
            Ok(()) => Ok(()),
            Err(e) => {
                error!("{e}");
                Err(e.into())
            }
        }
    }

    #[oai(path = "/bracket/:offset", method = "get")]
    async fn list_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<BracketGET>>> {
        match read_bracket(&db, offset.0) {
            Ok(brackets) => {
                let mut b_api_vec = vec![];
                for b in brackets {
                    let b_api: BracketGET = b.try_into()?;
                    b_api_vec.push(b_api);
                }
                Ok(Json(b_api_vec))
            }
            Err(e) => {
                error!("{e}");
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
    ) -> Result<Json<Vec<BracketGET>>> {
        match find_bracket(&db, bracket_name.0.as_str(), offset.0) {
            Ok(brackets) => {
                let mut b_api_vec = vec![];
                for b in brackets {
                    let b_api: BracketGET = b.try_into()?;
                    b_api_vec.push(b_api);
                }
                Ok(Json(b_api_vec))
            }
            Err(e) => {
                error!("{e}");
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
            Error::Denied() => pError::from_status(StatusCode::FORBIDDEN),
            Error::Unknown(_msg) => pError::from_status(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}

fn insert_bracket<'a, 'b, 'c>(db: &'a SharedDb, bracket_name: &'b str) -> Result<(), Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    db.create_bracket(bracket_name)?;
    Ok(())
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
