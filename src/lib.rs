#![deny(clippy::all, clippy::pedantic)]
#![warn(clippy::nursery)]
#![allow(
    clippy::missing_errors_doc,
    clippy::missing_panics_doc,
    clippy::must_use_candidate,
    clippy::redundant_pub_crate
)]

pub mod app;
pub mod config;
pub mod domain;
pub mod email_client;
pub mod routes;
pub mod telemetry;
