//! Bracket domain
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use totsugeki::{
    bracket::{Bracket, Id as BracketId},
    organiser::Id as OrganiserId,
    player::Id as PlayerId,
    DiscussionChannelId,
};

#[derive(Serialize, Deserialize, Object)]
/// POST request to /bracket
pub struct POST {
    /// name of the bracket
    pub bracket_name: String,
    /// name of the organiser if unknown to totsugeki
    pub organiser_name: String,
    /// id of the organiser specific to service used
    pub organiser_internal_id: String,
    /// id of discussion channel specific to service used
    pub channel_internal_id: String,
    /// name of service used to interact with api
    pub service_type_id: String, // TODO rename service_type_name
}

/// Bracket GET response
//
// NOTE: having Bracket implement `ToJSON` means that importing `totsugeki` will bring in all poem
// dependencies. This does not play nice with yew dependencies when doing relative import
// (totsugeki = { path = "../totsugeki" }) and caused many errors. The workaround is to leave
// Bracket package as barebones as possible and let packages importing it the task of deriving
// necessary traits into their own structs.
#[derive(Object, Serialize, Deserialize)]
pub struct GETResponse {
    bracket_id: BracketId,
    bracket_name: String,
    players: Vec<PlayerId>,
}

impl GETResponse {
    /// Form values to be sent to the API to create a bracket
    #[must_use]
    pub fn new(bracket: &Bracket) -> Self {
        GETResponse {
            bracket_id: bracket.get_id(),
            bracket_name: bracket.get_bracket_name(),
            players: bracket.get_players(),
        }
    }
}

impl From<Bracket> for GETResponse {
    fn from(b: Bracket) -> Self {
        GETResponse::new(&b)
    }
}

#[derive(Object)]
/// Bracket POST response body
pub struct POSTResult {
    /// bracket identifier
    pub bracket_id: BracketId,
    /// organiser identifier
    pub organiser_id: OrganiserId,
    /// discussion channel identifier
    pub discussion_channel_id: DiscussionChannelId,
}

impl POSTResult {
    #[must_use]
    /// Create response body
    pub fn new(
        bracket_id: BracketId,
        organiser_id: OrganiserId,
        discussion_channel_id: DiscussionChannelId,
    ) -> Self {
        Self {
            bracket_id,
            organiser_id,
            discussion_channel_id,
        }
    }
}

impl From<totsugeki::bracket::POSTResult> for POSTResult {
    fn from(tb: totsugeki::bracket::POSTResult) -> Self {
        Self {
            bracket_id: tb.get_bracket_id(),
            organiser_id: tb.get_organiser_id(),
            discussion_channel_id: tb.get_discussion_channel_id(),
        }
    }
}
