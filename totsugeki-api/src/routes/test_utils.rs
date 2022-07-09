//! Provide endpoint to setup/teardown database for test purposes. Disabled when production mode is enabled.

use crate::persistence::Error;
use crate::routes::SharedDb;
use crate::MyApiKeyAuthorization;
use poem::Result;
use poem_openapi::OpenApi;

/// Api calls to clean up database for testing purposes
pub struct TestUtilsApi;

#[OpenApi]
impl TestUtilsApi {
    #[oai(path = "/clean", method = "delete")]
    async fn clean_database<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: MyApiKeyAuthorization,
    ) -> Result<()> {
        // TODO find trait implementation to use ? instead of unwrapping
        if let Err(e) = clean_database(&db) {
            Err(e.into())
        } else {
            Ok(())
        }
    }
}

fn clean_database<'a, 'b>(db: &SharedDb<'a>) -> Result<(), Error<'b>>
where
    'a: 'b,
{
    let mut db = db.write()?;
    Ok(db.clean()?)
}
