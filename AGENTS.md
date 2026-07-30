# Contributor Instructions

Read `README.md`, `docs/PRODUCT_PRINCIPLES.md`, and `docs/ARCHITECTURE.md`
before making architecture or feature decisions.

## Boundaries

- Keep `nuofield-core` free of filesystem, network, database, and async-runtime
  dependencies.
- Validate domain policy before durable append; apply projections only after a
  successful append.
- Treat humans and agents as separate actors. Never let an agent inherit a
  human credential.
- Distinguish users from deployment operators. Infrastructure custody does not
  transfer ownership of user data or intelligent assets.
- Treat Fieldkeeper skills, memory, permissions, and behavior changes as
  versioned user-owned assets.
- Make model egress visible and removable. No external model is a runtime
  requirement.
- Store user data only under the configured operator-managed data directory.
- Keep public documentation self-contained and product-focused.

## Quality

Run before committing:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

Commits use `sail <sailcpu@icloud.com>` and describe the product change without
tool attribution.
