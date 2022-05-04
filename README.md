## Linker
lld for linux, zld for mac. Maybe try out mold?

## Watcher
```sh
cargo watch -x check -x test -x run
```

## Code coverage
```sh
cargo tarpaulin --ignore-tests
```

## Linting
```sh
cargo clippy -- -D warnings
```

## Formatting
```sh
cargo fmt
```

## Security Vulnerabilities
Maybe try cargo-deny?
```sh
cargo audit
```

## Unused dependencies
```sh
cargo +nightly udeps
```

## Run a specific test with logs
```sh
TEST_LOG=true cargo test health_check_works | bunyan
``` 
