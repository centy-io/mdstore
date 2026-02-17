# Contributing

## Prerequisites

- Rust toolchain (see `rust-version` in `Cargo.toml` for MSRV)
- `rustfmt` and `clippy` components installed

## Development

### Running checks locally

```sh
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
cargo build
```

### Lint configuration

The project enforces strict clippy lints defined in `Cargo.toml` including pedantic rules and denying panics, unwraps, unsafe code, and unchecked arithmetic/indexing. CI will fail if any of these are violated.

## CI

Every push and pull request to `main` runs:

- `cargo fmt --check`
- `cargo clippy` with project lint rules
- `cargo test`
- `cargo build`
- MSRV build and test (Rust 1.70)
- `cargo package` dry-run

## Publishing

Releases are published to crates.io automatically when a version tag (`v*.*.*`) is pushed.

### Setup

A `CARGO_REGISTRY_TOKEN` repository secret must be configured with a crates.io API token.

### Release process

1. Update the version in `Cargo.toml`
2. Commit and push to `main`
3. Tag the commit: `git tag v0.1.0`
4. Push the tag: `git push origin v0.1.0`

The publish workflow will validate the package, publish to crates.io, and create a GitHub Release with a changelog.
