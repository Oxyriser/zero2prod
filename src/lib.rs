#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::net::SocketAddr;

use axum::extract::Form;
use axum::routing::{get, post, IntoMakeService};
use axum::{Router, Server};
use hyper::server::conn::AddrIncoming;
use serde::Deserialize;

#[derive(Deserialize)]
struct FormData {
    email: String,
    name: String,
}

pub fn run(address: SocketAddr) -> Server<AddrIncoming, IntoMakeService<Router>> {
    let app = Router::new()
        .route("/health_check", get(|| async {}))
        .route("/subscriptions", post(subscribe));

    Server::bind(&address).serve(app.into_make_service())
}

async fn subscribe(_form: Form<FormData>) {}
