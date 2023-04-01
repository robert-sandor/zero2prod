use std::net::TcpListener;

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

/// spins up an instance of the application and returns its address
/// example: http://localhost:8080
fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("to find a random port to bind to");

    let port = listener
        .local_addr()
        .expect("to have a local address")
        .port();

    let server = zero2prod::run(listener).expect("to bind server to the address");

    let _ = tokio::spawn(server);

    format!("http://127.0.0.1:{port}")
}
