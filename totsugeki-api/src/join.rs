//! join request body and response

use poem_openapi::Object;
use totsugeki::{bracket::Id as BracketId, organiser::Id as OrganiserId, player::Id as PlayerId};

/// POST request to /join endpoint
#[derive(Object)]
pub struct POSTRequest {
    /// player id of service
    pub player_internal_id: String,
    /// name of unregistered player
    pub player_name: String,
    /// channel id of service
    pub channel_internal_id: String,
    /// service type
    pub service_type_id: String,
}

/// POST response to /join endpoint
#[derive(Object)]
pub struct POSTResponse {
    /// Player identifier
    pub player_id: PlayerId,
    /// Bracket identifier
    pub bracket_id: BracketId,
    /// Organiser identifier
    pub organiser_id: OrganiserId,
}
