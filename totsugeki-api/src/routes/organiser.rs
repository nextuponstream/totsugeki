//! Organiser Api

use crate::log_error;
use crate::persistence::Error;
use crate::ApiKeyServiceAuthorization;
use crate::SharedDb;
use poem::Result;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use totsugeki::organiser::{GETResponse, Organiser, POSTRequest};

/// Organiser Api
pub struct Api;

#[OpenApi]
impl Api {
    /// Create new organiser
    #[oai(path = "/organiser", method = "post")]
    #[tracing::instrument(name = "Create organiser", skip(self, db, _auth))]
    async fn create_organiser<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        organiser_request: Json<POSTRequest>,
    ) -> Result<()> {
        match register_organiser(&db, organiser_request.organiser_name.as_str()) {
            Ok(()) => Ok(()),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// List registered organisers
    #[oai(path = "/organiser/:offset", method = "get")]
    async fn list_organiser<'a>(
        &self,
        db: SharedDb<'a>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<GETResponse>>> {
        match list_registered_organisers(&db, offset.0) {
            Ok(os) => {
                let mut o_api_vec = vec![];
                for o in os {
                    o_api_vec.push(o.try_into()?);
                }

                Ok(Json(o_api_vec))
            }
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Find a specific organiser by name
    #[oai(path = "/organiser/:organiser_name/:offset", method = "get")]
    async fn find_organiser<'a>(
        &self,
        db: SharedDb<'a>,
        organiser_name: Path<String>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<GETResponse>>> {
        match find_organiser(&db, organiser_name.0.as_str(), offset.0) {
            Ok(os) => {
                let mut o_api_vec = vec![];
                for o in os {
                    o_api_vec.push(o.try_into()?);
                }

                Ok(Json(o_api_vec))
            }
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }
}

/// Call to register organiser made to database
fn register_organiser<'a, 'b, 'c>(
    db: &'a SharedDb,
    organiser_name: &'b str,
) -> Result<(), Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    db.create_organiser(organiser_name)?;
    Ok(())
}

/// List registered organiser
fn list_registered_organisers<'a, 'b>(
    db: &'a SharedDb,
    offset: i64,
) -> Result<Vec<Organiser>, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.list_organisers(offset)
}

/// Find registered organiser by `organiser_name`
fn find_organiser<'a, 'b, 'c>(
    db: &'a SharedDb,
    organiser_name: &'b str,
    offset: i64,
) -> Result<Vec<Organiser>, Error<'c>>
where
    'a: 'b,
    'b: 'c,
{
    let db = db.read()?;
    db.find_organisers(organiser_name, offset)
}
