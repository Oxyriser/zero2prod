#![deny(clippy::all)]
#![warn(clippy::pedantic)]
#![warn(clippy::nursery)]

use std::net::SocketAddr;

use axum::routing::get;
use axum::{Router, Server};

#[tokio::main]
async fn main() {
    let app = Router::new().route("/health_check", get(|| async {}));

    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("Listening on {}", addr);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
