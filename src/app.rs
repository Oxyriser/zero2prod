use std::net::SocketAddr;
use std::sync::Arc;

use axum::routing::{get, post, IntoMakeService};
use axum::{Extension, Router, Server};
use hyper::server::conn::AddrIncoming;
use sqlx::migrate::MigrateError;
use sqlx::PgPool;

use crate::routes::subscribe;

pub struct State {
    pub db_pool: PgPool,
}

pub fn run(address: SocketAddr, db_pool: PgPool) -> Server<AddrIncoming, IntoMakeService<Router>> {
    let state = Arc::new(State { db_pool });

    let app = Router::new()
        .route("/health_check", get(|| async {}))
        .route("/subscriptions", post(subscribe))
        .layer(Extension(state));

    Server::bind(&address).serve(app.into_make_service())
}

pub async fn run_migrations(db_pool: &PgPool) -> Result<(), MigrateError> {
    sqlx::migrate!("./migrations").run(db_pool).await
}
