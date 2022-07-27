//! join

use crate::join::{POSTRequest, POSTResponse};
use crate::persistence::Error;
use crate::{log_error, ApiKeyServiceAuthorization, SharedDb};
use poem::Result;
use poem_openapi::{payload::Json, OpenApi};
use totsugeki::join::POSTResponseBody;

/// Join Api
pub struct Api;

#[OpenApi]
impl Api {
    /// Let player join bracket from organiser
    #[oai(path = "/join", method = "post")]
    async fn join_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        join_request: Json<POSTRequest>,
    ) -> Result<Json<POSTResponse>> {
        match join(&db, &join_request.0) {
            Ok(r) => Ok(Json(r)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }
}

fn join<'a, 'b>(db: &'a SharedDb, j: &POSTRequest) -> Result<POSTResponse, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    let body = db.join_bracket(
        j.player_internal_id.as_str(),
        j.channel_internal_id.as_str(),
        j.service_type_id.as_str(),
    )?;
    let response: POSTResponse = body.into();
    Ok(response)
}

impl From<POSTResponseBody> for POSTResponse {
    fn from(b: POSTResponseBody) -> Self {
        Self {
            player_id: b.player_id,
            bracket_id: b.bracket_id,
            organiser_id: b.organiser_id,
        }
    }
}
