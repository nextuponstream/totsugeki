//! Utility function for integration testing
//! NOTE: other people also have the same idea, see [link](https://stackoverflow.com/a/59090848)

/// Test app with convenience methods to avoid test boilerplate
///
/// IDEA: [link](https://youtu.be/_VB1fLLtZfQ?t=961)
pub struct TestApp {
    /// http address of test app
    pub addr: String,
    /// http client with cookie jar to store sessions.
    http_client: Client,
}

use super::*;
use crate::brackets::{BracketState, CreateBracketForm};
use reqwest::{Client, Response};
use serde::Serialize;
use tokio::net::TcpListener;

/// Returns address to connect to new application (with random available port)
///
/// Example: `http://0.0.0.0:43222`
#[must_use]
#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
pub async fn spawn_app(db: PgPool) -> TestApp {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap())
        .await
        .unwrap();
    let addr = listener.local_addr().unwrap();
    let session_store = PostgresStore::new(db.clone());
    session_store.migrate().await.unwrap();

    let session_layer = SessionManagerLayer::new(session_store.clone())
        .with_secure(false)
        .with_expiry(Expiry::OnInactivity(Duration::hours(1)));

    tokio::spawn(async move {
        axum::serve(
            listener,
            app(db, session_store)
                .layer(session_layer)
                .into_make_service(),
        )
        .await
        .unwrap();
    });

    TestApp::new(format!("http://{addr}"))
}

/// User registration form input
#[derive(Serialize, Debug)]
pub struct FormUserInput {
    /// user name
    pub name: String,
    /// user email address
    pub email: String,
    /// user provided password
    pub password: String,
}

/// User registration form input
#[derive(Serialize, Debug)]
pub struct LoginForm {
    /// user email
    pub email: String,
    /// user provided password
    pub password: String,
}

impl TestApp {
    /// A connector
    #[must_use]
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub fn new(addr: String) -> TestApp {
        TestApp {
            addr,
            http_client: Client::builder().cookie_store(true).build().unwrap(),
        }
    }

    /// register user through `/api/register` endpoint with a POST request
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn register(&self, request: &FormUserInput) -> Response {
        self.http_client
            .post(format!("{}/api/register", self.addr))
            .json(request)
            .send()
            .await
            .expect("request done")
    }

    /// login user through `/api/login` endpoint with a POST request and store
    /// session in the http client of `TestApp` instance for further usage in
    /// future requests
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn login(&self, request: &LoginForm) -> Response {
        self.http_client
            .post(format!("{}/api/login", self.addr))
            .basic_auth(&request.email, Some(&request.password))
            .send()
            .await
            .expect("request done")
    }

    /// `/api/users DELETE` Delete user if logged in. User deleted is logged-in
    /// user. You must log in for this request to succeed.
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn delete_user(&self) -> Response {
        self.http_client
            .delete(format!("{}/api/users", self.addr))
            .send()
            .await
            .expect("request done")
    }

    /// Chains user registration and user login for a new user.
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn login_as_test_user(&self) {
        let response = self
            .register(&FormUserInput {
                name: "jean".into(),
                email: "jean@bon.ch".into(),
                password: "verySecurePassword#123456789?".into(),
            })
            .await;

        let status = response.status();
        assert!(
            status.is_success(),
            "status: {status}, response: \"{}\"",
            response.text().await.unwrap()
        );

        let response = self
            .login(&LoginForm {
                email: "jean@bon.ch".into(),
                password: "verySecurePassword#123456789?".into(),
            })
            .await;

        let status = response.status();
        assert!(
            status.is_success(),
            "status: {status}, response: \"{}\"",
            response.text().await.unwrap()
        );
    }

    /// `/api/brackets` POST
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn create_bracket(&self, players: Vec<String>) -> Response {
        let request = CreateBracketForm {
            bracket_name: "placeholder".into(),
            player_names: players,
        };
        self.http_client
            .post(format!("{}/api/brackets", self.addr))
            .json(&request)
            .send()
            .await
            .expect("request done")
    }

    /// `/api/brackets/:id` GET
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn get_bracket(&self, id: Id) -> Response {
        self.http_client
            .get(format!("{}/api/brackets/{id}", self.addr))
            .send()
            .await
            .expect("request done")
    }

    /// `/api/brackets` GET
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn list_brackets(&self, limit: u32, offset: u32) -> Response {
        self.http_client
            .get(format!(
                "{}/api/brackets?limit={}&offset={}&sort_order=DESC",
                self.addr, limit, offset
            ))
            .send()
            .await
            .expect("request done")
    }

    /// `/api/brackets/save` POST
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn save_bracket(&self, state: BracketState) -> Response {
        let request = state;
        self.http_client
            .post(format!("{}/api/brackets/save", self.addr))
            .json(&request)
            .send()
            .await
            .expect("request done")
    }
}
