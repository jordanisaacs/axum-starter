use std::net::TcpListener;

use axum_starter::{
    configuration::get_configuration,
    telemetry::{get_subscriber, init_subscriber},
};
use sqlx::PgPool;

/// Runs the axum server with configuration
///
/// Loads configuration, connects to database, initializes a subscriber,
/// binds to TCP address, then runs the server
#[tokio::main]
async fn main() -> hyper::Result<()> {
    let configuration;
    let connection_pool;
    let subscriber;
    let listener;
    let server;

    configuration = get_configuration().expect("Failed to read configuration.");
    connection_pool = PgPool::connect(&configuration.database.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    subscriber = get_subscriber("axum-starter".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    listener =
        TcpListener::bind(configuration.server.public_addr()).expect("Failed to bind to address");

    server = axum_starter::run(listener, connection_pool)?;

    server.await
}
