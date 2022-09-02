//! validate matches

use super::TotsugekiApiTestClient;
use totsugeki::matches::{Id as MatchId, MatchGET};
use tracing::{debug, trace};

/// validate match where predicted seed `x` and `y` play
pub async fn validate_match_for_predicted_seeds(
    test_api: &TotsugekiApiTestClient,
    x: usize,
    y: usize,
    matches: &Vec<MatchGET>,
) {
    debug!("Tournament organiser validates match for seed {x} and {y}");
    for m in matches {
        debug!("{m:?}");
    }
    let match_to_validate = matches
        .iter()
        .find(|m| m.seeds[0] == x && m.seeds[1] == y)
        .expect("Match to validate");
    let res = test_api
        .cli
        .post(format!("/bracket/validate/{}", match_to_validate.id))
        .header("X-API-Key", test_api.authorization_header.as_str())
        .send()
        .await;
    res.assert_status_is_ok();
    trace!("Match validated");
}

/// Validate match with `match_id`
/// validate match where predicted seed `x` and `y` play
pub async fn validate_match(test_api: &TotsugekiApiTestClient, match_id: MatchId) {
    let res = test_api
        .cli
        .post(format!("/bracket/validate/{}", match_id))
        .header("X-API-Key", test_api.authorization_header.as_str())
        .send()
        .await;
    res.assert_status_is_ok();
}
