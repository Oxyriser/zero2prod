use std::net::SocketAddr;

use once_cell::sync::Lazy;
use reqwest::Body;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::app::{build, get_db_pool};
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
    pub email_server: MockServer,
}

impl TestApp {
    pub async fn post_subscriptions<T: Into<Body>>(&self, body: T) -> reqwest::Response {
        reqwest::Client::new()
            .post(&format!("http://{}/subscriptions", &self.address))
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body)
            .send()
            .await
            .expect("Failed to execute request.")
    }
}

pub async fn spawn_app() -> TestApp {
    Lazy::force(&TRACING);

    let email_server = MockServer::start().await;

    let config = {
        let mut c = get_configuration().expect("Failed to read configuration.");
        c.database.database_name = Uuid::new_v4().to_string();
        c.application.port = 0;
        c.email_client.base_url = email_server.uri();
        c
    };

    configure_database(&config.database).await;
    let server = build(&config);
    let local_addr = server.local_addr();
    let _ = tokio::spawn(server);

    TestApp {
        address: local_addr,
        db_pool: get_db_pool(&config.database),
        email_server,
    }
}

async fn configure_database(config: &DatabaseSettings) -> PgPool {
    PgConnection::connect_with(&config.without_db())
        .await
        .expect("Failed to connect to Postgres")
        .execute(format!(r#"CREATE DATABASE "{}";"#, config.database_name).as_str())
        .await
        .expect("Failed to create database.");

    let db_pool = PgPool::connect_with(config.with_db())
        .await
        .expect("Failed to connect to Postgres.");

    sqlx::migrate!("./migrations")
        .run(&db_pool)
        .await
        .expect("Error running migrations.");

    db_pool
}
