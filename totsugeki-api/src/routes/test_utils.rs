//! Provide endpoint to setup/teardown database for test purposes

use crate::persistence::Error;
use crate::{ApiKeyServiceAuthorization, SharedDb};
use poem::Result;
use poem_openapi::OpenApi;

/// Api calls to clean up database for testing purposes
pub struct Api;

#[OpenApi]
impl Api {
    #[oai(path = "/clean", method = "delete")]
    async fn clean_database<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
    ) -> Result<()> {
        // TODO find trait implementation to use ? instead of unwrapping
        match clean_database(&db) {
            Err(e) => Err(e.into()),
            _ => Ok(()),
        }
    }
}

fn clean_database<'a, 'b>(db: &'a SharedDb) -> Result<(), Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.clean()?;
    Ok(())
}
