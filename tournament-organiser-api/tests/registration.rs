//! tests/health_check.rs
use reqwest::Client;
use tournament_organiser_api::registration::FormInput;
use tournament_organiser_api::test_utils::spawn_app;

#[cfg(test)]
#[tokio::test]
async fn registration() {
    let app = spawn_app().await;
    let client = Client::new();
    // TODO clean db before running test or spawn a new one

    let registration = FormInput {
        name: "jean".into(),
        email: "jean@bon.ch".into(),
        password: "verySecurePassword#123456789?".into(),
        created_at: None,
    };
    let response = client
        .post(format!("{}/api/register", app.addr))
        .json(&registration)
        .send()
        .await
        .expect("Failed to execute request.");

    let status = response.status();
    assert!(
        status.is_success(),
        "status: {status}, response: \"{}\"",
        response.text().await.unwrap()
    );
}
