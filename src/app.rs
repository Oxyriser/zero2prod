use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post, IntoMakeService};
use axum::{Extension, Router, Server};
use hyper::server::conn::AddrIncoming;
use hyper::{Body, Request};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;
use uuid::Uuid;

use crate::routes::subscribe;

pub struct State {
    pub db_pool: PgPool,
}

pub fn run(address: SocketAddr, db_pool: PgPool) -> Server<AddrIncoming, IntoMakeService<Router>> {
    let state = Arc::new(State { db_pool });

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
