//! Common setup/teardown methods

pub mod bracket;
pub mod disqualify;
pub mod forfeit;
pub mod join;
pub mod matches;
pub mod next_match;
pub mod players;
pub mod report;
pub mod validate;

use hmac::Hmac;
use poem::Route;
use sha2::Sha256;
use std::sync::Arc;
use totsugeki::ReadLock;
use totsugeki_api::{oai_test_service, persistence::DBAccessor, route_with_data, DatabaseType};

pub fn db_types_to_test() -> Vec<DatabaseType> {
    let db_types_in_test_arg =
        std::env::var("API_DB_TYPES_IN_TESTS").expect("API_DB_TYPES_IN_TESTS");
    let db_types_in_tests: Vec<&str> = db_types_in_test_arg.split(',').collect();
    let db_types_in_tests: Vec<DatabaseType> = db_types_in_tests
        .iter()
        .map(|skipped_db| skipped_db.parse::<DatabaseType>().expect("Database type"))
        .collect();
    if db_types_in_tests.is_empty() {
        panic!("At least one type of database has to be tested. Set API_DB_TYPES_IN_TESTS.")
    }
    db_types_in_tests
}

fn app_route() -> Route {
    Route::new().nest("/", oai_test_service())
}

type ApiTestClient = poem::test::TestClient<
    poem::middleware::AddDataEndpoint<
        poem::middleware::AddDataEndpoint<
            poem::middleware::CorsEndpoint<Route>,
            Arc<ReadLock<Box<dyn DBAccessor + Send + Sync>>>,
        >,
        Hmac<Sha256>,
    >,
>;

/// Return a test client
pub async fn test_api(db_type: DatabaseType) -> TotsugekiApiTestClient {
    let cli = poem::test::TestClient::new(route_with_data(
        app_route(),
        db_type,
        "123456".as_bytes(), // TODO generate random key
    ));

    let service_name = "hello"; // TODO generate random name
    let service_description = "desc"; // TODO generate random description
    let service_registration_endpoint_url =
        format!("/service/register/{service_name}/{service_description}");
    let resp = cli.post(service_registration_endpoint_url).send().await;
    resp.assert_status_is_ok();
    let authorization_header = resp
        .json()
        .await
        .value()
        .object()
        .get("token")
        .string()
        .to_string();

    TotsugekiApiTestClient {
        cli,
        authorization_header,
    }
}

pub struct TotsugekiApiTestClient {
    pub cli: ApiTestClient,
    pub authorization_header: String,
}

impl TotsugekiApiTestClient {
    /// Clean database
    pub async fn clean_db(&self) {
        let resp = self
            .cli
            .delete("/clean")
            .header("X-API-Key", self.authorization_header.as_str())
            .send()
            .await;
        resp.assert_status_is_ok();
    }
}
