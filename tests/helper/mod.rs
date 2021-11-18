use axum_starter::{
    configuration::{get_configuration, DatabaseSettings},
    run,
    telemetry::{get_subscriber, init_subscriber},
};
use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use std::net::TcpListener;
use uuid::Uuid;

// Ensure `tracing` stack is only initialized once using `once_cell`
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level;
    let subscriber_name;

    default_filter_level = "info".to_string();
    subscriber_name = "test".to_string();

    // Logs to stdout if `TEST_LOG` is set. If not set send into the void
    if std::env::var("TEST_LOG").is_ok() {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::stdout);
        init_subscriber(subscriber);
    } else {
        let subscriber = get_subscriber(subscriber_name, default_filter_level, std::io::sink);
        init_subscriber(subscriber);
    };
});

pub struct TestApp {
    pub address: String,
    pub db_pool: PgPool,
}

pub async fn spawn_app() -> TestApp {
    let listener;
    let port;
    let address;
    let mut configuration;
    let db_pool;
    let server;

    // Only invoke code if first time being called
    Lazy::force(&TRACING);

    listener = TcpListener::bind("127.0.0.1:0").expect("Failed to bind to random port");
    port = listener.local_addr().unwrap().port();
    address = format!("http://127.0.0.1:{}", port);

    configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();
    db_pool = configure_database(&configuration.database).await;

    server = run(listener, db_pool.clone()).expect("Failed to bind to address");
    let _ = tokio::spawn(server);
    TestApp { address, db_pool }
}

pub async fn configure_database(config: &DatabaseSettings) -> PgPool {
    let mut connection;
    let connection_pool;

    connection = PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres");

    connection
        .execute(format!(r#"create database "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    connection_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&connection_pool)
        .await
        .expect("Failed to migrate the database");

    connection_pool
}
