//! Utility function for integration testing
//! NOTE: other people also have the same idea, see [link](https://stackoverflow.com/a/59090848)

/// Test app with convenience methods to avoid test boilerplate
///
/// IDEA: [link](https://youtu.be/_VB1fLLtZfQ?t=961)
pub struct TestApp {
    /// http address of test app
    pub addr: String,
}

use crate::registration::FormInput;
use reqwest::{Client, Response};

use super::*;
use std::net::TcpListener;
/// Returns address to connect to new application (with random available port)
///
/// Example: `http://0.0.0.0:43222`
#[must_use]
#[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
pub async fn spawn_app(db: PgPool) -> TestApp {
    let listener = TcpListener::bind("0.0.0.0:0".parse::<SocketAddr>().unwrap()).unwrap();
    let addr = listener.local_addr().unwrap();

    tokio::spawn(async move {
        axum::Server::from_tcp(listener)
            .unwrap()
            .serve(app(db).into_make_service())
            .await
            .unwrap();
    });

    TestApp {
        addr: format!("http://{addr}"),
    }
}

impl TestApp {
    /// register user through `/api/register` endpoint with a POST request
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn register(&self, request: &FormInput) -> Response {
        let client = Client::new();
        client
            .post(format!("{}/api/register", self.addr))
            .json(request)
            .send()
            .await
            .expect("request done")
    }
}
