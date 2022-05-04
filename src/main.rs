use std::net::SocketAddr;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use zero2prod::app::{run, run_migrations};
use zero2prod::config::get_configuration;
use zero2prod::telemetry::setup_tracing;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    setup_tracing("zero2prod", "info", std::io::stdout);

    let config = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(5))
        .connect(&config.database.connection_string())
        .await
        .expect("Cannot connect to the database.");

    run_migrations(&db_pool)
        .await
        .expect("Error running migrations");

    let address = SocketAddr::from(([127, 0, 0, 1], config.application_port));
    run(address, db_pool).await?;

    Ok(())
}
