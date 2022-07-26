// test /organiser endpoint

mod common;

use common::{db_types_to_test, test_api};
use poem::http::StatusCode;

// Feature: Generating organiser when creating bracket
// Scenario: New organiser is generated when bracket is created
//  ✔  Given my-favorite-to wants to create a bracket named basel-weekly
//  ✔  When they create a bracket using discord bot
//  ✔  Then there is a organiser named FancyBar with the new active bracket
#[tokio::test]
async fn new_organiser_is_generated_when_bracket_is_created_if_unknown() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        todo!();
        // let resp = test_api.cli.post("/bracket").send().await;
        // resp.assert_status_is_ok();
        test_api.clean_db().await;
    }
}
// Scenario: Organiser run another bracket
//  ✔  Given my-favorite-to wants to create a bracket named basel-weekly-return
//  ✔  When the organiser FancyBar has already ran a bracket named basel-weekly
//  ✔  When they create a bracket using discord bot
//  ✔  Then there is only one organiser with two brackets named basel-weekly and basel-weekly-return
#[tokio::test]
async fn running_consecutive_brackets() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        todo!();
        // let resp = test_api.cli.post("/bracket").send().await;
        // resp.assert_status_is_ok();
        test_api.clean_db().await;
    }
}
