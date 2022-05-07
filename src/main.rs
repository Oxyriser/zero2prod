use std::net::SocketAddr;
use std::str::FromStr;
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use zero2prod::app::run;
use zero2prod::config::get_configuration;
use zero2prod::telemetry::setup_tracing;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    setup_tracing("zero2prod", "info", std::io::stdout);

    let config = get_configuration().expect("Failed to read configuration.");
    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(5))
        .connect_lazy_with(config.database.with_db());

    let address = SocketAddr::from_str(&format!(
        "{}:{}",
        config.application.host, config.application.port
    ))
    .expect("Invalid address");

    run(address, db_pool).await?;

    Ok(())
}
