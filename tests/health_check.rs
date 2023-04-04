use once_cell::sync::Lazy;
use secrecy::ExposeSecret;
use std::{collections::HashMap, net::TcpListener};

use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::config::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::{get_subscriber, init_subscriber};

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;

    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/health_check", test_app.address))
        .send()
        .await
        .expect("request to be succesful");

    assert!(response.status().is_success());
    assert_eq!(Some(0), response.content_length());
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let mut form = HashMap::new();
    form.insert("name", "test name");
    form.insert("email", "testname@gmail.com");

    let response = client
        .post(format!("{}/subscriptions", test_app.address))
        .form(&form)
        .send()
        .await
        .expect("request to be succesful");

    assert_eq!(200, response.status().as_u16());

    let saved = sqlx::query!("select email, name from subscriptions")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("to read a subscription");

    assert_eq!(saved.email, "testname@gmail.com");
    assert_eq!(saved.name, "test name");
}

#[tokio::test]
async fn subscribe_returns_a_400_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        (Some("test name"), None, "missing the email"),
        (None, Some("testname@gmail.com"), "missing the name"),
        (None, None, "missing both name and email"),
    ];

    for (name, email, _error_message) in test_cases {
        let mut form = HashMap::new();
        form.insert("name", name);
        form.insert("email", email);

        let response = client
            .post(format!("{}/subscriptions", test_app.address))
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
    }
}

#[tokio::test]
async fn subscribe_returns_400_with_bad_or_empty_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("test name", "", "missing the email"),
        ("", "testname@gmail.com", "missing the name"),
        ("", "", "missing both name and email"),
    ];

    for (name, email, _error_message) in test_cases {
        let mut form = HashMap::new();
        form.insert("name", name);
        form.insert("email", email);

        let response = client
            .post(format!("{}/subscriptions", test_app.address))
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
    }
}

static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    if std::env::var("TEST_LOG").is_ok() {
        init_subscriber(get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::stdout,
        ))
    } else {
        init_subscriber(get_subscriber(
            subscriber_name,
            default_filter_level,
            std::io::sink,
        ))
    }
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

/// spins up an instance of the application and returns its address
/// example: http://localhost:8080
async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let listener = TcpListener::bind("127.0.0.1:0").expect("to find a random port to bind to");

    let port = listener
        .local_addr()
        .expect("to have a local address")
        .port();
    let address = format!("http://127.0.0.1:{port}");

    let mut config = get_configuration().expect("to read configuration");
    config.database.database_name = Uuid::new_v4().to_string();
    let db_pool = configure_database(&config.database).await;
    let server =
        zero2prod::startup::run(listener, db_pool.clone()).expect("to bind server to the address");

    let _ = tokio::spawn(server);

    TestApp { address, db_pool }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection = PgConnection::connect(&config.connection_string_no_db().expose_secret())
        .await
        .expect("to connect to the database");

    connection
        .execute(format!(r#"create database "{}""#, config.database_name).as_str())
        .await
        .expect("to be able to create the database");

    let db_pool = PgPool::connect(&config.connection_string().expose_secret())
        .await
        .expect("to be able to connect to the database");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("to be able to migrate the database");

    db_pool
}
