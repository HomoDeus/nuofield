# Contributing to NuoField

Thanks for helping build an open, self-hosted workspace for humans and agents.

## Development setup

Requirements:

- Rust 1.88 or newer
- Docker 24 or newer for container validation

```bash
git clone https://github.com/HomoDeus/nuofield.git
cd nuofield
cargo test --workspace
```

## Before opening a pull request

Run the complete local gate:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace --release
```

Changes to domain behavior should include tests in `nuofield-core`. Changes to
durability or audit behavior should include tests in `nuofield-store`.

## Architecture rules

- Domain policy lives in `nuofield-core` and performs no I/O.
- Durable append happens before state projection.
- Every privileged action identifies its actor.
- High-risk actions fail closed until a human approves them.
- Model calls and evidence remain exportable.
- External services are optional adapters, not boot requirements.

## Pull requests

Keep each pull request focused. Explain:

- what behavior changes;
- why the change belongs in the current milestone;
- which security or data boundary is affected;
- how the change was tested.

By contributing, you agree that your contribution is licensed under the MIT
License.
