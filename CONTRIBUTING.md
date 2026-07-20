# Contributing

## Prerequisites

- Rust toolchain (see `rust-version` in `Cargo.toml` for MSRV)
- `rustfmt` and `clippy` components installed
- [pnpm](https://pnpm.io/) (for Git hooks)

## Setup

```sh
pnpm install
```

This automatically runs `husky` via the `prepare` script to install the Git hooks.

## Git hooks

This project uses [Husky](https://typicode.github.io/husky/) to enforce code quality via Git hooks and [commitlint](https://commitlint.js.org/) to enforce [Conventional Commits](https://www.conventionalcommits.org/).

| Hook | Runs | Purpose |
|---|---|---|
| `pre-commit` | `cargo fmt --check`, `cargo clippy -- -D warnings` | Prevent unformatted or lint-failing code from being committed |
| `pre-push` | `cargo test` | Prevent pushing code that breaks tests |
| `commit-msg` | `commitlint` | Enforce Conventional Commits format |

### Commit message format

All commit messages must follow the [Conventional Commits](https://www.conventionalcommits.org/) specification:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

Common types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`.

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
- MSRV build and test (Rust 1.85)
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
