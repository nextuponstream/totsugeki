// test /join endpoint

mod common;

use common::{db_types_to_test, test_api};

// Feature: Join bracket as player
//   Scenario: Many players join a bracket
//    ✔  Given my-favorite-to has created a bracket named basel-weekly
//    ✔  When the-new-lad, the-old-time-player and 6 other players join
//    ✔  Then there is enough people for an 8 participants tournament
#[tokio::test]
async fn player_joins_bracket() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        todo!();
        // let resp = test_api.cli.post("/bracket").send().await;
        // resp.assert_status_is_ok();
        test_api.clean_db().await;
    }
}
