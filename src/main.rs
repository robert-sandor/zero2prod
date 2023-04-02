use std::net::TcpListener;
use zero2prod::{config::get_configuration, startup::run};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let config = get_configuration().expect("configuration to be read");
    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port))?;
    run(listener)?.await
}
