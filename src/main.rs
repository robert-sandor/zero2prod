use secrecy::ExposeSecret;
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

    tracing::info!(
        "Starting server on host {} port {}",
        config.app.host,
        config.app.port
    );

    let connection = PgPool::connect_lazy(&config.database.connection_string().expose_secret())
        .expect("to connect to the databse");

    let listener = TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))?;
    run(listener, connection)?.await
}
