//! Http responses to bracket queries

use crate::{
    bracket::Id, matches::MatchGET, organiser::Id as OrganiserId, player::Player,
    DiscussionChannelId,
};
#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::{Deserialize, Serialize};

use super::raw::Raw;

#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
/// POST request to /bracket endpoint
pub struct POST {
    /// name of the bracket
    pub bracket_name: String,
    /// used to create missing organiser
    pub organiser_name: String,
    /// Identifier of the organiser from the service (for instance: discord)
    pub organiser_internal_id: String,
    /// Identifier of the discussion channel from service (for instance: discord)
    pub channel_internal_id: String,
    /// Name of service. See totsugeki_api for a list of supported service
    pub service_type_id: String,
    /// bracket format
    pub format: String,
    /// seeding method for bracket
    pub seeding_method: String,
    /// Advertised start time
    pub start_time: String,
    /// Automatically validate match if both players agree
    pub automatic_match_validation: bool,
}

/// Bracket GET response
#[derive(Serialize, Deserialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
#[cfg_attr(feature = "poem-openapi", oai(rename = "BracketGET"))]
pub struct GET {
    /// Identifier of bracket
    pub bracket_id: Id,
    /// Name of this bracket
    pub bracket_name: String,
    /// Players in this bracket
    pub players: Vec<Player>,
    /// Matches for this bracket
    pub matches: Vec<MatchGET>,
    /// Bracket format
    pub format: String,
    /// Seeding method used for this bracket
    pub seeding_method: String,
    /// Advertised start time
    pub start_time: String,
    /// Accept match results
    pub accept_match_results: bool,
    /// Automatically validate match results if both players agree
    pub automatic_match_validation: bool,
    /// Bar new participants from entering
    pub barred_from_entering: bool,
}

impl GET {
    /// Form values to be sent to the API to create a bracket
    #[must_use]
    pub fn new(bracket: &Raw) -> Self {
        GET {
            bracket_id: bracket.bracket_id,
            bracket_name: bracket.bracket_name.clone(),
            players: bracket.get_players_list(),
            format: bracket.format.to_string(),
            seeding_method: bracket.seeding_method.to_string(),
            matches: bracket
                .matches
                .clone()
                .into_iter()
                .map(std::convert::Into::into)
                .collect::<Vec<MatchGET>>(),
            start_time: bracket.start_time.to_string(),
            accept_match_results: bracket.accept_match_results,
            automatic_match_validation: bracket.automatic_match_validation,
            barred_from_entering: bracket.barred_from_entering,
        }
    }
}

/// POST response to /bracket endpoint
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct POSTResult {
    /// id of created bracket
    pub bracket_id: Id,
    /// id of organiser
    pub organiser_id: OrganiserId,
    /// id of discussion channel
    pub discussion_channel_id: DiscussionChannelId,
}

/// POST request body for interacting with a bracket, like closing or starting
/// the bracket
#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct CommandPOST {
    /// Discussion channel id of service
    pub channel_internal_id: String,
    /// Service used to make call
    pub service_type_id: String,
}

/// POST response to /bracket/start endpoint
#[derive(Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
pub struct StartPOSTResult {
    /// id of created bracket
    pub bracket_id: Id,
    /// Matches to play
    pub matches: Vec<MatchGET>,
}
