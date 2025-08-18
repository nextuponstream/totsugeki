//! Utility function for integration testing
//! NOTE: other people also have the same idea, see [link](https://stackoverflow.com/a/59090848)

/// Test app with convenience methods to avoid test boilerplate
///
/// IDEA: [link](https://youtu.be/_VB1fLLtZfQ?t=961)
pub struct TestApp {
    /// http address of test app
    pub addr: String,
    /// http client with cookie jar to store sessions.
    pub http_client: Client,
}

use super::{app, Expiry, PgPool, PostgresStore, SessionManagerLayer, SocketAddr};
use crate::tournaments::{BracketState, CreateTournamentForm, ReportResultInput};
use crate::ID;
use reqwest::{Client, Response};
use serde::Serialize;
use time::Duration;
use tokio::net::TcpListener;
use totsugeki_core::matches::result::Score;

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
        let url = format!("{}/api/register", self.addr);
        self.http_client
            .post(&url)
            .json(request)
            .send()
            .await
            .unwrap_or_else(|err| panic!("POST request to {url}: {err}"))
    }

    /// login user through `/api/login` endpoint with a POST request and store
    /// session in the http client of `TestApp` instance for further usage in
    /// future requests
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn login(&self, request: &LoginForm) -> Response {
        let url = format!("{}/api/login", self.addr);
        self.http_client
            .post(&url)
            .basic_auth(&request.email, Some(&request.password))
            .send()
            .await
            .unwrap_or_else(|err| panic!("POST request to {url}: {err}"))
    }

    /// `/api/users DELETE` Delete user if logged in. User deleted is logged-in
    /// user. You must log in for this request to succeed.
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn delete_user(&self) -> Response {
        let url = format!("{}/api/users", self.addr);
        self.http_client
            .delete(&url)
            .send()
            .await
            .unwrap_or_else(|err| panic!("DELETE request to {url}: {err}"))
    }

    /// Chains user registration and user login for a new user.
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn login_as_test_user(&self) {
        let response = self
            .login(&LoginForm {
                email: "test@user.ch".into(),
                password: "securePass123#".into(),
            })
            .await;

        let status = response.status();
        assert!(
            status.is_success(),
            "status: {status}, response: \"{}\"",
            response.text().await.unwrap()
        );
    }

    /// `/api/tournaments` POST
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn create_tournament(&self, players: Vec<String>) -> Response {
        let request = CreateTournamentForm {
            tournament_name: "placeholder".into(),
            player_names: players,
        };
        let url = format!("{}/api/tournaments", self.addr);
        self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .unwrap_or_else(|err| panic!("POST request to {url}: {err}"))
    }

    /// `/api/tournaments/:id` GET
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn get_tournament(&self, id: ID) -> Response {
        let url = format!("{}/api/tournaments/{id}", self.addr);
        self.http_client
            .get(&url)
            .send()
            .await
            .unwrap_or_else(|err| panic!("GET request to {url}: {err}"))
    }

    /// `/api/tournaments` GET
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn list_brackets(&self, limit: u32, offset: u32) -> Response {
        let url = format!(
            "{}/api/tournaments?limit={}&offset={}&sort_order=DESC",
            self.addr, limit, offset
        );
        self.http_client
            .get(&url)
            .send()
            .await
            .unwrap_or_else(|err| panic!("GET request to {url}: {err}"))
    }

    /// `/api/tournaments/save` POST
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn save_tournament(&self, state: BracketState) -> Response {
        let request = state;
        let url = format!("{}/api/tournaments/save", self.addr);
        self.http_client
            .post(&url)
            .json(&request)
            .send()
            .await
            .unwrap_or_else(|err| panic!("POST request to {url}: {err}"))
    }

    /// `/api/tournaments/:bracket_id/join` POST
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn join_tournament(&self, tournament_id: ID) -> Response {
        let url = format!("{}/api/tournaments/{}/join", self.addr, tournament_id);
        self.http_client
            .post(&url)
            .send()
            .await
            .unwrap_or_else(|err| panic!("POST request to {url}: {err}"))
    }

    /// `/api/tournaments/:tournament_id/report` POST
    #[allow(clippy::unwrap_used, clippy::missing_panics_doc)]
    pub async fn report_score_for_tournament(
        &self,
        tournament_id: &ID,
        score: Score,
        high_seed_player_id: &ID,
        low_seed_player_id: &ID,
    ) -> Response {
        let url = format!("{}/api/tournaments/{}/score", self.addr, tournament_id);
        // FIXME when sending 0-0, should respond 400
        self.http_client
            .post(&url)
            .json(&ReportResultInput {
                player1_id: high_seed_player_id.to_owned(),
                player2_id: low_seed_player_id.to_owned(),
                score_p1: score.0,
                score_p2: score.1,
            })
            .send()
            .await
            .unwrap_or_else(|err| panic!("POST request to {url}: {err}"))
    }
}
