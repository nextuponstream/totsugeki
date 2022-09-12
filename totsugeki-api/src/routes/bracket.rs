//! bracket routes
use crate::log_error;
use crate::persistence::Error;
use crate::ApiKeyServiceAuthorization;
use crate::SharedDb;
use crate::GET as BracketGET;
use poem::Result;
use poem_openapi::param::Path;
use poem_openapi::payload::Json;
use poem_openapi::OpenApi;
use totsugeki::bracket::{CommandPOST, POST};
use totsugeki::matches::ReportResultPOST;
use totsugeki::player::PlayersRaw;
use totsugeki::{
    bracket::{CreateRequest, Id as BracketId, POSTResult, Raw},
    matches::{Id as MatchId, MatchResultPOST, NextMatchGETRequest, NextMatchGETResponseRaw},
    player::{Player, GET as PlayersGET},
    quit::POST as QuitPOST,
    remove::POST as RemovePOST,
    seeding::POST as SeedPOST,
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
        let db_request = CreateRequest {
            bracket_name: r.bracket_name.as_str(),
            bracket_format: r.format.as_str(),
            seeding_method: r.seeding_method.as_str(),
            organiser_name: r.organiser_name.as_str(),
            organiser_internal_id: r.organiser_internal_id.as_str(),
            internal_channel_id: r.channel_internal_id.as_str(),
            service_type_id: r.service_type_id.as_str(),
            start_time: r.start_time.as_str(),
            automatic_match_validation: r.automatic_match_validation,
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
    ) -> Result<Json<BracketGET>> {
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
    ) -> Result<Json<Vec<BracketGET>>> {
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
    ) -> Result<Json<Vec<BracketGET>>> {
        info!("{}, {}", bracket_name.0, offset.0);
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
    ) -> Result<Json<ReportResultPOST>> {
        match report(&db, &r.0) {
            Ok(response) => Ok(Json(response)),
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

    /// Start bracket (accept results) and return id of bracket affected
    #[oai(path = "/bracket/start", method = "post")]
    #[tracing::instrument(name = "Start bracket", skip(self, db, _auth))]
    async fn start_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<CommandPOST>,
    ) -> Result<Json<BracketId>> {
        match start(&db, &r.0) {
            Ok(bracket_id) => Ok(Json(bracket_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Return players in active bracket
    ///
    /// Useful to issue before seeding.
    #[oai(path = "/bracket/players", method = "get")]
    #[tracing::instrument(name = "List players", skip(self, db))]
    async fn list_players<'a>(
        &self,
        db: SharedDb<'a>,
        r: Json<PlayersGET>,
    ) -> Result<Json<PlayersRaw>> {
        match list_players_in_bracket(&db, &r.0) {
            Ok(players) => Ok(Json(players)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Return bracket id after seeding
    #[oai(path = "/bracket/seed", method = "post")]
    #[tracing::instrument(name = "Seed bracket", skip(self, db, _auth))]
    async fn seed<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<SeedPOST>,
    ) -> Result<Json<BracketId>> {
        match seed_bracket(&db, r.0) {
            Ok(bracket_id) => Ok(Json(bracket_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Close bracket (prevent new participants from entering) and return id of
    /// affected bracket
    #[oai(path = "/bracket/close", method = "post")]
    #[tracing::instrument(name = "Close bracket", skip(self, db, _auth))]
    async fn close_bracket<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<CommandPOST>,
    ) -> Result<Json<BracketId>> {
        match close(&db, &r.0) {
            Ok(bracket_id) => Ok(Json(bracket_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Quit bracket bracket and return id of affected bracket
    #[oai(path = "/bracket/quit", method = "post")]
    #[tracing::instrument(name = "Quit bracket", skip(self, db, _auth))]
    async fn quit<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<QuitPOST>,
    ) -> Result<Json<BracketId>> {
        match quit_bracket(&db, &r.0) {
            Ok(bracket_id) => Ok(Json(bracket_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Remove player from bracket and return id of affected bracket
    #[oai(path = "/bracket/remove", method = "post")]
    #[tracing::instrument(name = "Remove player from bracket", skip(self, db, _auth))]
    async fn remove<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<RemovePOST>,
    ) -> Result<Json<BracketId>> {
        match remove_player(&db, &r.0) {
            Ok(bracket_id) => Ok(Json(bracket_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Remove player from bracket and return id of affected bracket
    #[oai(path = "/bracket/disqualify", method = "post")]
    #[tracing::instrument(name = "Disqualify player from bracket", skip(self, db, _auth))]
    async fn disqualify<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<RemovePOST>,
    ) -> Result<Json<BracketId>> {
        match disqualify_player(&db, &r.0) {
            Ok(bracket_id) => Ok(Json(bracket_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }

    /// Forfeit in bracket and return id of affected bracket
    #[oai(path = "/bracket/forfeit", method = "post")]
    #[tracing::instrument(name = "Forfeit bracket", skip(self, db, _auth))]
    async fn forfeit<'a>(
        &self,
        db: SharedDb<'a>,
        _auth: ApiKeyServiceAuthorization,
        r: Json<QuitPOST>,
    ) -> Result<Json<BracketId>> {
        match forfeit_bracket(&db, &r.0) {
            Ok(bracket_id) => Ok(Json(bracket_id)),
            Err(e) => {
                log_error(&e);
                Err(e.into())
            }
        }
    }
}

/// Database call to create new active bracket from issued discussion channel
fn create_new_active_bracket<'a, 'b, 'c>(
    db: &'a SharedDb,
    r: CreateRequest<'b>,
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
fn get_bracket<'a, 'b>(db: &'a SharedDb, bracket_id: BracketId) -> Result<Raw, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.get_bracket(bracket_id)
}

/// Call to database to list registered bracket
fn list_brackets<'a, 'b>(db: &'a SharedDb, offset: i64) -> Result<Vec<Raw>, Error<'b>>
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
) -> Result<Vec<Raw>, Error<'c>>
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
fn report<'a, 'b>(db: &'a SharedDb, r: &MatchResultPOST) -> Result<ReportResultPOST, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.report_result(
        &r.internal_player_id,
        &r.internal_channel_id,
        &r.service,
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

/// Update bracket to start accepting match results
fn start<'a, 'b>(db: &'a SharedDb, r: &CommandPOST) -> Result<BracketId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.start_bracket(&r.channel_internal_id, &r.service_type_id)
}

/// List players in bracket
fn list_players_in_bracket<'a, 'b>(
    db: &'a SharedDb,
    r: &PlayersGET,
) -> Result<PlayersRaw, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    let (bracket_id, players) = db.list_players(r)?;
    let players = players.get_players_list();
    let players_ids = players.iter().map(Player::get_id).collect();
    let players_names = players.iter().map(Player::get_name).collect();
    Ok(PlayersRaw {
        bracket_id,
        players: players_ids,
        player_names: players_names,
    })
}

/// Seed players in bracket
fn seed_bracket<'a, 'b>(db: &'a SharedDb, r: SeedPOST) -> Result<BracketId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.seed_bracket(&r.internal_channel_id, &r.service, r.players)
}

/// Update bracket by closing it
fn close<'a, 'b>(db: &'a SharedDb, r: &CommandPOST) -> Result<BracketId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.close_bracket(&r.channel_internal_id, &r.service_type_id)
}

/// Let player quit bracket
fn quit_bracket<'a, 'b>(db: &'a SharedDb, r: &QuitPOST) -> Result<BracketId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.quit_bracket(&r.internal_channel_id, &r.service, &r.internal_player_id)
}

/// Remove player from bracket
fn remove_player<'a, 'b>(db: &'a SharedDb, r: &RemovePOST) -> Result<BracketId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.remove_player(&r.internal_channel_id, &r.service, &r.player_id)
}

/// Disqualify player from bracket
fn disqualify_player<'a, 'b>(db: &'a SharedDb, r: &RemovePOST) -> Result<BracketId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.disqualify_player(&r.internal_channel_id, &r.service, &r.player_id)
}

/// Let player quit bracket
fn forfeit_bracket<'a, 'b>(db: &'a SharedDb, r: &QuitPOST) -> Result<BracketId, Error<'b>>
where
    'a: 'b,
{
    let db = db.read()?;
    db.forfeit(&r.internal_channel_id, &r.service, &r.internal_player_id)
}
