//! bracket routes
use crate::log_error;
use crate::persistence::{BracketRequest, Error};
use crate::ApiKeyServiceAuthorization;
use crate::SharedDb;
use crate::GET;
use poem::Result;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use totsugeki::bracket::POST;
use totsugeki::{
    bracket::{Bracket, Id as BracketId, POSTResult},
    matches::{Id as MatchId, MatchResultPOST, NextMatchGETRequest, NextMatchGETResponseRaw},
};
use tracing::info;

/// Bracket Api
pub struct Api;

#[OpenApi]
impl Api {
    /// Create a new active bracket in issued discussion channel
    #[oai(path = "/bracket", method = "post")]
    #[tracing::instrument(name = "Creating new bracket", skip(self, db, _auth))]
    async fn create_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<POST>,
    ) -> Result<Json<POSTResult>> {
        let db_request = BracketRequest {
            bracket_name: r.bracket_name.as_str(),
            bracket_format: r.format.as_str(),
            seeding_method: r.seeding_method.as_str(),
            organiser_name: r.organiser_name.as_str(),
            organiser_internal_id: r.organiser_internal_id.as_str(),
            internal_channel_id: r.channel_internal_id.as_str(),
            service_type_id: r.service_type_id.as_str(),
            start_time: r.start_time.as_str(),
        };
        match create_new_active_bracket(&db, db_request) {
            Ok(r) => Ok(Json(r)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Get specific bracket
    #[oai(path = "/bracket/:bracket_id", method = "get")]
    #[tracing::instrument(name = "Viewing bracket", skip(self, db, bracket_id))]
    async fn get_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        bracket_id: Path<BracketId>,
    ) -> Result<Json<GET>> {
        info!("{}", bracket_id.0);
        match get_bracket(&db, bracket_id.0) {
            Ok(bracket) => Ok(Json(bracket.try_into()?)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// List registered brackets
    #[oai(path = "/brackets/:offset", method = "get")]
    #[tracing::instrument(name = "Viewing bracket list", skip(self, db, offset))]
    async fn list_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<GET>>> {
        info!("{}", offset.0);
        match list_brackets(&db, offset.0) {
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
    #[oai(path = "/brackets/:bracket_name/:offset", method = "get")]
    #[tracing::instrument(name = "Find bracket by name", skip(self, db, bracket_name, offset))]
    async fn find_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        bracket_name: Path<String>,
        offset: Path<i64>,
    ) -> Result<Json<Vec<GET>>> {
        info!("{}, {}", bracket_name.0, offset.0);
        match find_bracket(&db, bracket_name.0.as_str(), offset.0) {
            Ok(brackets) => {
                let mut b_api_vec = vec![];
                for b in brackets {
                    let b_api: GET = b.try_into()?;
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

    /// Return opponent of next match
    #[oai(path = "/next_match", method = "get")]
    #[tracing::instrument(name = "Find next match", skip(self, db, _auth))]
    async fn next_match<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<NextMatchGETRequest>,
    ) -> Result<Json<NextMatchGETResponseRaw>> {
        match next_match_for_player(
            &db,
            r.player_internal_id.as_str(),
            r.channel_internal_id.as_str(),
            r.service_type_id.as_str(),
        ) {
            Ok(response) => Ok(Json(response)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Report result of bracket match
    #[oai(path = "/bracket/report", method = "post")]
    #[tracing::instrument(name = "Report match result", skip(self, db, _auth))]
    async fn report_match_result<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<MatchResultPOST>,
    ) -> Result<Json<MatchId>> {
        match report(&db, &r.0) {
            Ok(m_id) => Ok(Json(m_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Validate result of bracket by TO
    #[oai(path = "/bracket/validate/:id", method = "post")]
    #[tracing::instrument(name = "Validate match result", skip(self, db, _auth, id))]
    async fn validate_match_result<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        id: Path<MatchId>,
    ) -> Result<()> {
        info!("{}", id.0);
        if let Err(e) = validate(&db, id.0) {
            log_error(&e);
            Err(e.into())
        } else {
            Ok(())
        }
    }
}

/// Database call to create new active bracket from issued discussion channel
fn create_new_active_bracket<'a, 'b, 'c>(
    db: &'a SharedDb,
    r: BracketRequest<'b>,
) -> Result<POSTResult, Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    let result = db.create_bracket(r)?;
    Ok(result)
}

/// Call to database to list registered bracket
fn get_bracket<'a, 'b>(db: &'a SharedDb, bracket_id: BracketId) -> Result<Bracket, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.get_bracket(bracket_id)
}

/// Call to database to list registered bracket
fn list_brackets<'a, 'b>(db: &'a SharedDb, offset: i64) -> Result<Vec<Bracket>, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.list_brackets(offset)
}

/// Call to database to find bracket named `bracket_name`
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

/// Returns next match for player
fn next_match_for_player<'a, 'b, 'c>(
    db: &'a SharedDb,
    player_internal_id: &'b str,
    channel_internal_id: &'b str,
    service_type_id: &'b str,
) -> Result<NextMatchGETResponseRaw, Error<'c>>
where
    'a: 'c,
    'b: 'c,
{
    let db = db.read()?;
    db.find_next_match(player_internal_id, channel_internal_id, service_type_id)
}

/// Report match result
fn report<'a, 'b>(db: &'a SharedDb, r: &MatchResultPOST) -> Result<MatchId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.report_result(
        &r.player_internal_id,
        &r.channel_internal_id,
        &r.service_type_id,
        &r.result,
    )
}

/// Validate match result
fn validate<'a, 'b>(db: &'a SharedDb, match_id: MatchId) -> Result<(), Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.validate_result(match_id)
}
