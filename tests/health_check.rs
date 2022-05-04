use std::net::SocketAddr;

use once_cell::sync::Lazy;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use zero2prod::app::{run, run_migrations};
use zero2prod::config::{get_configuration, DatabaseSettings};
use zero2prod::telemetry::setup_tracing;

static TRACING: Lazy<()> = Lazy::new(|| {
    let name = "test";
    let env_filter = "debug";
    if std::env::var("TEST_LOG").is_ok() {
        setup_tracing(name, env_filter, std::io::stdout);
    } else {
        setup_tracing(name, env_filter, std::io::sink);
    };
});

pub struct TestApp {
    pub address: SocketAddr,
    pub db_pool: PgPool,
}

async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);
    let mut configuration = get_configuration().expect("Failed to read configuration.");
    configuration.database.database_name = Uuid::new_v4().to_string();

    let db_pool = configure_database(&configuration.database).await;

    let address = SocketAddr::from(([127, 0, 0, 1], 0));
    let server = run(address, db_pool.clone());

    let binded_address = server.local_addr();
    tokio::spawn(server);

    TestApp {
        address: binded_address,
        db_pool,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    PgConnection::connect(&config.connection_string_without_db())
        .await
        .expect("Failed to connect to Postgres")
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let db_pool = PgPool::connect(&config.connection_string())
        .await
        .expect("Failed to connect to Postgres.");

    run_migrations(&db_pool)
        .await
        .expect("Error running migrations");

    db_pool
}

#[tokio::test]
async fn health_check_works() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("http://{}/health_check", test_app.address))
        .send()
        .await
        .expect("Failed to execute request.");

    assert!(response.status().is_success());
    assert_eq!(response.content_length(), Some(0));
}

#[tokio::test]
async fn subscribe_returns_a_200_for_valid_form_data() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();

    let body = "name=le%20guin&email=ursula_le_guin%40gmail.com";
    let response = client
        .post(&format!("http://{}/subscriptions", &test_app.address))
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .expect("Failed to execute request.");

    assert_eq!(201, response.status().as_u16());

    let saved = sqlx::query!("SELECT email, name FROM subscription")
        .fetch_one(&test_app.db_pool)
        .await
        .expect("Failed to fetch saved subscription.");

    assert_eq!(saved.email, "ursula_le_guin@gmail.com");
    assert_eq!(saved.name, "le guin");
}

#[tokio::test]
async fn subscribe_returns_a_422_when_data_is_missing() {
    let test_app = spawn_app().await;
    let client = reqwest::Client::new();
    let test_cases = vec![
        ("name=le%20guin", "missing the email"),
        ("email=ursula_le_guin%40gmail.com", "missing the name"),
        ("", "missing both name and email"),
    ];
    for (invalid_body, error_message) in test_cases {
        let response = client
            .post(&format!("http://{}/subscriptions", &test_app.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(invalid_body)
            .send()
            .await
            .expect("Failed to execute request.");

        assert_eq!(
            422,
            response.status().as_u16(),
            "The API did not fail with 422 Unprocessable Entity when the payload was {}.",
            error_message
        );
    }
}
