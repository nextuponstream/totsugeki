//! join domain

use crate::{bracket::Id as BracketId, organiser::Id as OrganiserId, player::Id as PlayerId};
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

/// /join POST response body
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct POSTResponseBody {
    /// Player identifier
    pub player_id: PlayerId,
    /// bracket identifier
    pub bracket_id: BracketId,
    /// organiser identifier
    pub organiser_id: OrganiserId,
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
/// Join POST request body
pub struct POSTRequestBody {
    /// Player internal id
    pub player_internal_id: String,
    /// player_name: String,
    pub player_name: String,
    /// channel internal id of service
    pub channel_internal_id: String,
    /// Service type identifier
    pub service_type_id: String,
}

impl POSTRequestBody {
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
