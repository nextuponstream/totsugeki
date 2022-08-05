//! validate matches

use super::TotsugekiApiTestClient;
use totsugeki::matches::{Id as MatchId, MatchGET};

/// validate match where predicted seed `x` and `y` play
pub async fn validate_match_for_predicted_seeds(
    test_api: &TotsugekiApiTestClient,
    x: usize,
    y: usize,
    matches: Vec<Vec<MatchGET>>,
) {
    // NOTE: I don't know how to use for_each with async
    for round in &matches {
        for m in round {
            if m.seeds[0] == x && m.seeds[1] == y {
                let res = test_api
                    .cli
                    .post(format!("/bracket/validate/{}", m.id))
                    .header("X-API-Key", test_api.authorization_header.as_str())
                    .send()
                    .await;
                res.assert_status_is_ok();
                break;
            }
        }
    }
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
