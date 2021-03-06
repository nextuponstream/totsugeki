//! Organiser Api

use crate::persistence::Error;
use crate::ApiKeyServiceAuthorization;
use crate::OrganiserGETResponse;
use crate::OrganiserPOSTResponse;
use crate::SharedDb;
use log::warn;
use poem::Result;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use totsugeki::organiser::Organiser;

/// Organiser Api
pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/organiser", method = "post")]
    async fn create_organiser<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        organiser_request: Json<OrganiserPOSTResponse>,
    ) -> Result<()> {
        match insert_organiser(&db, organiser_request.organiser_name.as_str()) {
            Ok(()) => Ok(()),
            Err(e) => {
                warn!("{e}");
                Err(e.into())
            }
        }
    }

    #[oai(path = "/organiser/:offset", method = "get")]
    async fn list_organiser<'a>(
        &self,
        db: SharedDb<'a>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<OrganiserGETResponse>>> {
        match read_organiser(&db, offset.0) {
            Ok(os) => {
                let mut o_api_vec = vec![];
                for o in os {
                    o_api_vec.push(o.try_into()?);
                }

                Ok(Json(o_api_vec))
            }
            Err(e) => {
                warn!("{e}");
                Err(e.into())
            }
        }
    }

    #[oai(path = "/organiser/:organiser_name/:offset", method = "get")]
    async fn find_organiser<'a>(
        &self,
        db: SharedDb<'a>,
        organiser_name: Path<String>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<OrganiserGETResponse>>> {
        match find_organiser(&db, organiser_name.0.as_str(), offset.0) {
            Ok(os) => {
                let mut o_api_vec = vec![];
                for o in os {
                    o_api_vec.push(o.try_into()?);
                }

                Ok(Json(o_api_vec))
            }
            Err(e) => {
                warn!("{e}");
                Err(e.into())
            }
        }
    }
}

fn insert_organiser<'a, 'b, 'c>(db: &'a SharedDb, organiser_name: &'b str) -> Result<(), Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    db.create_organiser(organiser_name)?;
    Ok(())
}

fn read_organiser<'a, 'b>(db: &'a SharedDb, offset: i64) -> Result<Vec<Organiser>, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.list_organisers(offset)
}

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
