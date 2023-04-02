use std::{collections::HashMap, net::TcpListener};

use sqlx::{Connection, PgConnection};
use zero2prod::config::get_configuration;

#[tokio::test]
async fn health_check_works() {
    let address = spawn_app();

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{address}/health_check"))
        .send()
        .await
        .expect("request to be succesful");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let address = spawn_app();
    let config = get_configuration().expect("valid configuration");
    let connection_string = config.database.connection_string();
    let mut connection = PgConnection::connect(&connection_string)
        .await
        .expect("connect to postgresql");
    let client = reqwest::Client::new();

    let mut form = HashMap::new();
    form.insert("name", "test name");
    form.insert("email", "testname@gmail.com");

    let response = client
        .post(format!("{address}/subscriptions"))
        .form(&form)
        .send()
        .await
        .expect("request to be succesful");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions")
        .fetch_one(&mut connection)
        .await
        .expect("to read a subscription");

    assert_eq!(saved.email, "testname@gmail.com");
    assert_eq!(saved.name, "test name");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let address = spawn_app();
    let client = reqwest::Client::new();
    let test_cases = vec![
        (Some("test name"), None, "missing the email"),
        (None, Some("testname@gmail.com"), "missing the name"),
        (None, None, "missing both name and email"),
    ];

    for (name, email, error_message) in test_cases {
        let mut form = HashMap::new();
        form.insert("name", name);
        form.insert("email", email);

        let response = client
            .post(format!("{address}/subscriptions"))
            .form(&form)
            .send()
            .await
            .expect("request to be succesful");

        assert_eq!(
            400,
            response.status().as_u16(),
            "the api did not fail with 400 Bad Request with payload={:?}",
            form
        );

        // let response_body = std::str::from_utf8(
        //     response
        //         .bytes()
        //         .await
        //         .expect("to have a response body")
        //         .(),
        // );
        // assert_eq!(
        //     error_message, response_body,
        //     "the api did not return the expected error message '{}' got '{}' instead",
        //     error_message, response_body
        // );
    }
}

/// spins up an instance of the application and returns its address
/// example: http://localhost:8080
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("to find a random port to bind to");

    let port = listener
        .local_addr()
        .expect("to have a local address")
        .port();

    let server = zero2prod::startup::run(listener).expect("to bind server to the address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
