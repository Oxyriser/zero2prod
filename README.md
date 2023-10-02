# Zero2Axum

## Run the app
```sh
docker compose up -d
cargo sqlx migrate run
RUST_LOG=info cargo run
```

## Run a specific test with logs
```sh
TEST_LOG=true cargo test health_check_works | bunyan
``` 
