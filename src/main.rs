use zero2prod::{app::build, config::get_configuration, telemetry::setup_tracing};

#[tokio::main]
async fn main() -> hyper::Result<()> {
    setup_tracing("zero2prod", "info", std::io::stdout);

    let config = get_configuration().expect("Failed to read configuration.");
    build(config).await?;

    Ok(())
}
