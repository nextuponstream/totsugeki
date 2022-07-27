//! test /health_check

// NOTE: shared test modules needs to be public so dead code warning are not
// raised when using module but not all function are used
pub mod common;

use common::{db_types_to_test, test_api};

#[tokio::test]
async fn health_check_works() {
    for db_type in db_types_to_test() {
        let test_api = test_api(db_type).await;
        let resp = test_api.cli.get("/health_check").send().await;
        resp.assert_status_is_ok();
        test_api.clean_db().await;
    }
}
