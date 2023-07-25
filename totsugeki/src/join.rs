//! join domain

use crate::{bracket::Id as BracketId, player::Id as PlayerId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
#[cfg_attr(feature = "poem-openapi", oai(rename = "JoinPOSTResponse"))]
/// /join POST response body
pub struct POSTResponse {
    /// Player identifier
    pub player_id: PlayerId,
    /// bracket identifier
    pub bracket_id: BracketId,
}

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
#[cfg_attr(feature = "poem-openapi", oai(rename = "JoinPOST"))]
/// Join POST request body
pub struct POST {
    /// Player internal id
    pub player_internal_id: String,
    /// player_name: String,
    pub player_name: String,
    /// channel internal id of service
    pub channel_internal_id: String,
    /// Service type identifier
    pub service_type_id: String,
}

impl POST {
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
