use std::net::SocketAddr;

use linkify::{LinkFinder, LinkKind};
use once_cell::sync::Lazy;
use reqwest::Body;
use sqlx::{Connection, Executor, PgConnection, PgPool};
use uuid::Uuid;
use wiremock::MockServer;
use zero2prod::{
    app::{build, get_db_pool},
    config::{get_configuration, DatabaseSettings},
    telemetry::setup_tracing,
};

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

    let db_pool = get_db_pool(&config.database);

    configure_database(&config.database).await;
    let server = build(config);
    let local_addr = server.local_addr();
    let _ = tokio::spawn(server);

    TestApp {
        address: local_addr,
        db_pool,
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

pub struct ConfirmationLinks {
    pub html: reqwest::Url,
    pub plain_text: reqwest::Url,
}

pub fn get_confirmation_links(
    address: &SocketAddr,
    email_request: &wiremock::Request,
) -> ConfirmationLinks {
    let body: serde_json::Value = serde_json::from_slice(&email_request.body).unwrap();
    let get_link = |s: &str| {
        let links: Vec<_> = LinkFinder::new().kinds(&[LinkKind::Url]).links(s).collect();
        assert_eq!(links.len(), 1);
        let raw_link = links[0].as_str().to_owned();

        let mut confirmation_link = reqwest::Url::parse(&raw_link).unwrap();
        assert_eq!(
            confirmation_link.host_str().unwrap(),
            &address.ip().to_string()
        );
        confirmation_link.set_port(Some(address.port())).unwrap();
        confirmation_link
    };

    let html = get_link(&body["HtmlBody"].as_str().unwrap());
    let plain_text = get_link(&body["TextBody"].as_str().unwrap());

    ConfirmationLinks { html, plain_text }
}
