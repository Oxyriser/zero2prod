use std::{
    net::{IpAddr, SocketAddr},
    sync::Arc,
};

use axum::{
    routing::{get, post, IntoMakeService},
    Extension, Router, Server,
};
use hyper::server::conn::AddrIncoming;
use sqlx::postgres::{PgPool, PgPoolOptions};
use tower_http::{
    request_id::{MakeRequestUuid, PropagateRequestIdLayer, SetRequestIdLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

use crate::{
    config::{DatabaseSettings, Settings},
    email_client::EmailClient,
    routes::{confirm, subscribe},
};

pub struct State {
    pub db_pool: PgPool,
    pub email_client: EmailClient,
    pub base_url: String,
}

pub fn run(
    address: SocketAddr,
    db_pool: PgPool,
    email_client: EmailClient,
    base_url: String,
) -> Server<AddrIncoming, IntoMakeService<Router>> {
    let state = Arc::new(State {
        db_pool,
        email_client,
        base_url,
    });

    let app = Router::new()
        .route("/health_check", get(|| async {}))
        .route("/subscriptions", post(subscribe))
        .route("/subscriptions/confirm", get(confirm))
        .layer(Extension(state))
        .layer(PropagateRequestIdLayer::x_request_id())
        .layer(
            TraceLayer::new_for_http()
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(
                    DefaultOnResponse::new()
                        .level(Level::INFO)
                        .include_headers(true),
                )
                .make_span_with(
                    DefaultMakeSpan::new()
                        .level(Level::INFO)
                        .include_headers(true),
                ),
        )
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid));

    Server::bind(&address).serve(app.into_make_service())
}

pub fn build(config: Settings) -> Server<AddrIncoming, IntoMakeService<Router>> {
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

    run(address, db_pool, email_client, config.application.base_url)
}

pub fn get_db_pool(configuration: &DatabaseSettings) -> PgPool {
    PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.with_db())
}
