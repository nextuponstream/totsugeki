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
        axum::serve(listener, app(db).layer(session_layer).into_make_service())
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

    /// `/api/user DELETE` Delete user if logged in. User deleted is logged in
    /// user. You must login for this request to succeed.
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn delete_user(&self) -> Response {
        self.http_client
            .delete(format!("{}/api/user", self.addr))
            .send()
            .await
            .expect("request done")
    }
}
