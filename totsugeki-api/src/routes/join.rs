//! join

use crate::{log_error, persistence::Error, ApiKeyServiceAuthorization, SharedDb};
use poem::Result;
use poem_openapi::{payload::Json, OpenApi};
use totsugeki::join::{POSTRequestBody, POSTResponseBody};

/// Join Api
pub struct Api;

#[OpenApi]
impl Api {
    /// Let player join active bracket in issued discussion channel
    #[oai(path = "/join", method = "post")]
    #[tracing::instrument(name = "Join bracket", skip(self, db, _auth))]
    async fn join_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        join_request: Json<POSTRequestBody>,
    ) -> Result<Json<POSTResponseBody>> {
        match join(&db, &join_request.0) {
            Ok(r) => Ok(Json(r)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }
}

/// Call to database for player to join active bracket in discussion channel
fn join<'a, 'b>(db: &'a SharedDb, j: &POSTRequestBody) -> Result<POSTResponseBody, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    let response = db.join_bracket(
        j.player_internal_id.as_str(),
        j.player_name.as_str(),
        j.channel_internal_id.as_str(),
        j.service_type_id.as_str(),
    )?;
    Ok(response)
}
