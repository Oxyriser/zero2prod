use std::net::{IpAddr, SocketAddr};
use std::time::Duration;

use sqlx::postgres::PgPoolOptions;
use zero2prod::app::run;
use zero2prod::config::get_configuration;
use zero2prod::email_client::EmailClient;
use zero2prod::telemetry::setup_tracing;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    setup_tracing("zero2prod", "info", std::io::stdout);

    let config = get_configuration().expect("Failed to read configuration.");

    let ip: IpAddr = config.application.host.parse().expect("Invalid host");
    let address = SocketAddr::from((ip, config.application.port));

    let db_pool = PgPoolOptions::new()
        .max_connections(5)
        .connect_timeout(Duration::from_secs(5))
        .connect_lazy_with(config.database.with_db());

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
        config.email_client.authorization_token,
        timeout,
    );

    run(address, db_pool, email_client).await?;

    Ok(())
}
