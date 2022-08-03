//! Matches domain redefinition

use poem_openapi::Object;
use serde::Serialize;
use totsugeki::{bracket::Id as BracketId, matches::Id as MatchId};

/// REDEFINITION: response to next match query
#[derive(Object)]
pub struct NextMatchGET {
    /// Next opponent
    pub opponent: String,
    /// Id of next match
    pub match_id: MatchId,
    /// Bracket where next match happens
    pub bracket_id: BracketId,
}

/// REDEFINITION: request for next match
#[derive(Object, Serialize)]
pub struct NextMatchGETRequest {
    /// Next opponent
    pub player_internal_id: String,
    /// Identifier of the discussion channel from service (for instance: discord)
    pub channel_internal_id: String,
    /// Name of service. See totsugeki_api for a list of supported service
    pub service_type_id: String,
}
