//! join request body and response

use poem_openapi::Object;
use totsugeki::{bracket::BracketId, organiser::OrganiserId, PlayerId};

/// Join POST request body
#[derive(Object)]
pub struct JoinPOST {
    /// player id of service
    pub player_internal_id: String,
    /// name of unregistered player
    pub player_name: String,
    /// channel id of service
    pub channel_internal_id: String,
    /// service type
    pub service_type_id: String,
}

/// Join POST response body
#[derive(Object)]
pub struct JoinPOSTResponse {
    /// Player identifier
    pub player_id: PlayerId,
    /// Bracket identifier
    pub bracket_id: BracketId,
    /// Organiser identifier
    pub organiser_id: OrganiserId,
}
