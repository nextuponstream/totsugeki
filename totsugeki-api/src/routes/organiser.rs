//! Organiser Api

use crate::persistence::Error;
use crate::MyApiKeyAuthorization;
use crate::OrganiserPOST;
use crate::SharedDb;
use log::error;
use poem::Result;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;

/// Organiser Api
pub struct OrganiserApi;

#[OpenApi]
impl OrganiserApi {
    #[oai(path = "/organiser", method = "post")]
    async fn create_organiser<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: MyApiKeyAuthorization,
        organiser_request: Json<OrganiserPOST>,
    ) -> Result<()> {
        match insert_organiser(&db, organiser_request.organiser_name.as_str()) {
            Ok(()) => Ok(()),
            Err(e) => {
                error!("{e}");
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
