use secrecy::ExposeSecret;
use sqlx::postgres::PgPoolOptions;
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

    let db_pool = PgPoolOptions::new()
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy(&config.database.connection_string().expose_secret())
        .expect("create postgresql connection pool");

    let listener = TcpListener::bind(format!("{}:{}", config.app.host, config.app.port))?;
    run(listener, db_pool)?.await
}
