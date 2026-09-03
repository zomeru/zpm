# Contributing to zpm

Thanks for considering a contribution. This guide makes the local workflow quick to discover.

## Requirements

- **Rust** `1.85` or later (MSRV). The repository pins the development toolchain via `rust-toolchain.toml` (`stable` + `rustfmt` + `clippy`). `rustup` will install it automatically on first `cargo` invocation. Verify with `rustc --version`.
- **Optional tools** (for full local CI parity):
  - [`just`](https://github.com/casey/just) — task runner (`cargo install just` or `brew install just`)
  - [`lefthook`](https://github.com/evilmartians/lefthook) — Git hooks (`brew install lefthook` or `go install github.com/evilmartians/lefthook@latest`)
  - [`cargo-deny`](https://github.com/EmbarkStudios/cargo-deny) / [`cargo-audit`](https://github.com/RustSec/rustsec) / [`cargo-outdated`](https://github.com/kbknapp/cargo-outdated) — dependency auditing

## Bootstrap

```bash
git clone https://github.com/zomeru/zpm
cd zpm

# toolchain is auto-installed via rust-toolchain.toml
rustup show

# install optional helpers (only needed for `just` / hooks / audit)
cargo install just cargo-deny cargo-audit cargo-outdated
brew install lefthook  # or: go install github.com/evilmartians/lefthook@latest

# one-time hook setup (hooks are not required — CI enforces the same checks)
lefthook install      # or: just hooks
```

## Running zpm locally

```bash
# help
cargo run -- --help
# with arguments (note `--` separates cargo args from zpm args)
cargo run -- add react --dry-run
cargo run -- --pm pnpm install --dry-run
cargo run -- run dev -- --port 3000

# with `just`
just dev --help
just run add react --dry-run
```

## Common tasks

All tasks are plain `cargo` commands; `just` wraps the same commands for convenience.

| Goal | `cargo` | `just` |
|------|---------|--------|
| format | `cargo fmt --all` | `just format` |
| format check (CI) | `cargo fmt --all -- --check` | `just format-check` |
| check | `cargo check --all-targets --all-features` | `just check` |
| lint (Clippy, warnings denied) | `cargo clippy --all-targets --all-features -- -D warnings` | `just lint` |
| test (unit + integration) | `cargo test --all-features` | `just test` |
| test + doc tests | `cargo test --all-features && cargo test --doc` | `just test-all` |
| single test | `cargo test --all-features filter -- --nocapture` | `just test-one filter` |
| debug build | `cargo build` | `just build` |
| release build | `cargo build --release` | `just build-release` |
| full CI validation | see below | `just ci` |

Shortcuts from `.cargo/config.toml` are also available:

```bash
cargo format        # alias for `cargo fmt --all`
cargo format-check  # alias for `cargo fmt --all -- --check`
cargo lint          # alias for clippy with -D warnings
cargo test-all      # alias for `cargo test --all-features`
cargo build-release # alias for `cargo build --release`
```

## Full validation (CI equivalent)

```bash
just ci
# equivalent to:
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --doc
cargo build
```

CI (`.github/workflows/ci.yml`) runs the *same* underlying cargo commands on Ubuntu, macOS, and Windows, plus `cargo deny check advisories`.

## Git hooks

`lefthook.yml` defines:

- **pre-commit** (fast, parallel): `cargo fmt --check` + `cargo check`
- **pre-push** (thorough): `cargo clippy` + `cargo test`

Install after cloning: `lefthook install`. Hooks are tracked (`lefthook.yml`) and are *not* a hard requirement — CI will catch issues if you skip them. To bypass once: `git commit --no-verify`.

## Dependency management

```bash
cargo tree                # or: just tree
cargo outdated            # or: just outdated
cargo audit               # or: just audit
cargo deny check          # or: just deny
cargo update              # bump Cargo.lock within semver
cargo update -p <crate>   # bump single crate
```

For security/licensing, see `deny.toml`. Keep `Cargo.lock` committed for a binary.

## MSRV

The `rust-version` in `Cargo.toml` is `1.85` (edition 2024). CI verifies it on Ubuntu with toolchain `1.85`. Bump it only when a dependency or language feature genuinely requires a newer compiler, and update `rust-toolchain.toml` / `CONTRIBUTING.md` / CI accordingly.

## Project layout

```
src/
  main.rs              # thin entry point
  app.rs               # orchestration (testable via lib)
  cli/                 # Clap types + resolver
  config/              # TOML/ini + env precedence
  detection/           # lockfile / packageManager walk
  package_manager/     # Agent enum + central command tables
  process/             # argv-preserving execution
  ui/                  # colors / prompts (NO_COLOR aware)
  workspace/           # workspace root + package.json helpers
  catalog/             # pnpm / Yarn / Bun catalog providers
  fs.rs, error.rs
```

## Code style

- `rustfmt` is canonical; do not hand-format.
- `clippy -- -D warnings` must pass.
- Prefer `&str` / `String` / `Path` / `PathBuf` appropriately; avoid unnecessary clones.
- Keep `main.rs` thin; business logic lives in `lib.rs`/`app.rs` for testability.
- No `unwrap`/`expect` in production paths without a proven invariant (tests may use them).
- No `unsafe` — the crate should remain safe Rust (the only `unsafe` is the `std::env::set_var` wrapper required since Rust 1.85).
- Keep `pub` / `pub(crate)` intentional; prefer low coupling.

## Testing

```bash
cargo test --all-features   # fast
cargo test --doc            # doc examples
```

Tests live in `tests/` and inline `#[cfg(test)]`. No network or shell `sh -c` in tests.

## Releasing

Tagged pushes (`v*`) trigger `.github/workflows/release.yml`, which builds release binaries for 6 targets (Linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64/aarch64) and creates a GitHub Release.
