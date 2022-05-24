use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;

use axum::routing::{get, post, IntoMakeService};
use axum::{Extension, Router, Server};
use hyper::server::conn::AddrIncoming;
use hyper::{Body, Request};
use sqlx::postgres::{PgPool, PgPoolOptions};
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::config::{DatabaseSettings, Settings};
use crate::email_client::EmailClient;
use crate::routes::subscribe;

pub struct State {
    pub db_pool: PgPool,
    pub email_client: EmailClient,
}

pub fn run(
    address: SocketAddr,
    db_pool: PgPool,
    email_client: EmailClient,
) -> Server<AddrIncoming, IntoMakeService<Router>> {
    let state = Arc::new(State {
        db_pool,
        email_client,
    });

    let app = Router::new()
        .route("/health_check", get(|| async {}))
        .route("/subscriptions", post(subscribe))
        .layer(Extension(state))
        .layer(
            TraceLayer::new_for_http().make_span_with(|request: &Request<Body>| {
                let request_id = Uuid::new_v4();
                tracing::debug_span!(
                    "request",
                    %request_id,
                    method = %request.method(),
                    uri = %request.uri(),
                    version = ?request.version(),
                )
            }),
        );

    Server::bind(&address).serve(app.into_make_service())
}

pub fn build(config: &Settings) -> Server<AddrIncoming, IntoMakeService<Router>> {
    let ip: IpAddr = config.application.host.parse().expect("Invalid host");
    let address = SocketAddr::from((ip, config.application.port));

    let db_pool = get_db_pool(&config.database);

    let base_url = config
        .email_client
        .base_url
        .parse()
        .expect("Invalid url for email client.");
    let sender_email = config
        .email_client
        .sender()
        .expect("Invalid sender email address.");
    let timeout = config.email_client.timeout();
    let email_client = EmailClient::new(
        base_url,
        sender_email,
        config.email_client.authorization_token.clone(),
        timeout,
    );

    run(address, db_pool, email_client)
}

pub fn get_db_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
