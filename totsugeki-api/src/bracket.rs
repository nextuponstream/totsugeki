//! Bracket domain
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use totsugeki::{
    bracket::{Bracket, Id as BracketId},
    organiser::Id as OrganiserId,
    player::Id as PlayerId,
    DiscussionChannelId,
};

#[derive(Object, Serialize, Deserialize)]
/// REDEFINITION: POST request to /bracket endpoint
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
    /// bracket format
    pub format: String,
    /// seeding method for bracket
    pub seeding_method: String,
}

/// REDEFINITION: Bracket GET response
//
// NOTE: having Bracket implement `ToJSON` means that importing `totsugeki` will bring in all poem
// dependencies. This does not play nice with yew dependencies when doing relative import
// (totsugeki = { path = "../totsugeki" }) and caused many errors. The workaround is to leave
// Bracket package as barebones as possible and let packages importing it the task of deriving
// necessary traits into their own structs.
#[derive(Object, Serialize, Deserialize)]
pub struct GETResponse {
    /// Identifier of bracket
    bracket_id: BracketId,
    /// Name of this bracket
    bracket_name: String,
    /// Players in this bracket
    players: Vec<PlayerId>,
    /// Matches for this bracket
    matches: Vec<Vec<Match>>,
    /// Bracket format
    format: String,
    /// Seeding method used for this bracket
    seeding_method: String,
}

/// REDEFINITION: A match between two players, resulting in a winner and a loser
#[derive(Object, Debug, Default, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Match {
    /// Two players from this match. One of the player can be a BYE opponent
    players: [String; 2],
    /// seeds\[0\]: top seed
    /// seeds\[1\]: bottom seed
    seeds: [usize; 2],
    /// The winner of this match
    winner: String,
    /// The looser of this match
    looser: String,
}

impl Match {
    /// Get matches to send
    fn get_sendable_matches(matches: &Vec<Vec<totsugeki::matches::Match>>) -> Vec<Vec<Match>> {
        let mut result = vec![];
        for round in matches {
            let mut result_round = vec![];
            for m in round {
                result_round.push(m.into());
            }

            result.push(result_round);
        }

        result
    }
}

impl GETResponse {
    /// Form values to be sent to the API to create a bracket
    #[must_use]
    pub fn new(bracket: &Bracket) -> Self {
        GETResponse {
            bracket_id: bracket.get_id(),
            bracket_name: bracket.get_bracket_name(),
            players: bracket.get_players(),
            format: bracket.get_format().to_string(),
            seeding_method: bracket.get_seeding_method().to_string(),
            matches: Match::get_sendable_matches(&bracket.get_matches()),
        }
    }
}

impl From<&totsugeki::matches::Match> for Match {
    fn from(m: &totsugeki::matches::Match) -> Self {
        let player_1 = m.get_players()[0].to_string();
        let player_2 = m.get_players()[1].to_string();
        Self {
            players: [player_1, player_2],
            seeds: m.get_seeds(),
            winner: m.get_winner().to_string(),
            looser: m.get_looser().to_string(),
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
