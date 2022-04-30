use std::net::SocketAddr;
use zero2prod::run;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    run(address).await?;
    Ok(())
}
