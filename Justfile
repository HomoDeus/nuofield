set dotenv-load := true

default:
    @just --list

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint:
    cargo clippy --workspace --all-targets -- -D warnings

test:
    cargo test --workspace

build:
    cargo build --workspace

build-release:
    cargo build --workspace --release

ci: fmt-check lint test build-release

run:
    cargo run -p nuofield-server

id:
    cargo run -p nuofield-cli --bin nuofield -- id

docker:
    docker build -t nuofield:dev .
