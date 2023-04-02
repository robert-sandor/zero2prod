use sqlx::PgPool;
use std::net::TcpListener;
use zero2prod::telemetry::get_subscriber;
use zero2prod::telemetry::init_subscriber;
use zero2prod::{
    config::{get_configuration, Settings},
    startup::run,
};

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    init_subscriber(get_subscriber(
        "zero2prod".into(),
        "info".into(),
        std::io::stdout,
    ));
    let config: Settings = get_configuration().expect("configuration to be read");

    let connection = PgPool::connect(&config.database.connection_string())
        .await
        .expect("to connect to the databse");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", config.port))?;
    run(listener, connection)?.await
}
