//! join

use crate::join::{JoinPOST, JoinPOSTResponse};
use crate::persistence::Error;
use crate::{log_error, ApiKeyServiceAuthorization, SharedDb};
use poem::Result;
use poem_openapi::{payload::Json, OpenApi};
use totsugeki::join::JoinPOSTResponseBody;

/// Join Api
pub struct JoinApi;

#[OpenApi]
impl JoinApi {
    /// Let player join bracket from organiser
    #[oai(path = "/join", method = "post")]
    async fn join_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        join_request: Json<JoinPOST>,
    ) -> Result<Json<JoinPOSTResponse>> {
        match join(&db, join_request.0) {
            Ok(r) => Ok(Json(r)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }
}

fn join<'a, 'b>(db: &'a SharedDb, j: JoinPOST) -> Result<JoinPOSTResponse, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    let body = db.join_bracket(
        j.player_internal_id.as_str(),
        j.channel_internal_id.as_str(),
        j.service_type_id.as_str(),
    )?;
    let response: JoinPOSTResponse = body.into();
    Ok(response)
}

impl From<JoinPOSTResponseBody> for JoinPOSTResponse {
    fn from(b: JoinPOSTResponseBody) -> Self {
        Self {
            player_id: b.player_id,
            bracket_id: b.bracket_id,
            organiser_id: b.organiser_id,
        }
    }
}
