//! join domain

use serde::{Deserialize, Serialize};

use crate::{bracket::BracketId, organiser::OrganiserId, PlayerId};

/// Join POST response body
#[derive(Debug, Deserialize)]
pub struct JoinPOSTResponseBody {
    /// Player identifier
    pub player_id: PlayerId,
    /// bracket identifier
    pub bracket_id: BracketId,
    /// organiser identifier
    pub organiser_id: OrganiserId,
}

#[derive(Serialize)]
/// Join POST request body
pub struct JoinPOSTRequestBody {
    /// Player internal id
    player_internal_id: String,
    /// player_name: String,
    player_name: String,
    /// channel internal id of service
    channel_internal_id: String,
    /// Service type identifier
    service_type_id: String,
}

impl JoinPOSTRequestBody {
    /// Create new Join POST request body
    #[must_use]
    pub fn new(
        player_internal_id: String,
        player_name: String,
        channel_internal_id: String,
        service_type_id: String,
    ) -> Self {
        Self {
            player_internal_id,
            player_name,
            channel_internal_id,
            service_type_id,
        }
    }
}
