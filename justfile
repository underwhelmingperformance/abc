default: check

build:
    cargo build

run *args:
    cargo run -- {{args}}

test:
    cargo test

clippy:
    cargo clippy --all-targets --all-features -- -D warnings

fmt:
    cargo-nightly fmt --all

check: test clippy fmt
