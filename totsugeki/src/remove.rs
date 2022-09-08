//! As a TO, remove player from bracket before bracket start

#[cfg(feature = "poem-openapi")]
use poem_openapi::Object;
use serde::Serialize;

#[derive(Serialize, Debug)]
#[cfg_attr(feature = "poem-openapi", derive(Object))]
#[cfg_attr(feature = "poem-openapi", oai(rename = "BracketRemovePOST"))]
/// Remove POST request body
pub struct POST {
    /// channel internal id of service
    pub internal_channel_id: String,
    /// Player internal id
    pub player_id: String,
    /// Service type identifier
    pub service: String,
}
