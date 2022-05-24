use zero2prod::app::build;
use zero2prod::config::get_configuration;
use zero2prod::telemetry::setup_tracing;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    setup_tracing("zero2prod", "info", std::io::stdout);

    let config = get_configuration().expect("Failed to read configuration.");
    build(&config).await?;

    Ok(())
}
