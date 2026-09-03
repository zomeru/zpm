# zpm developer tasks — run with `just <task>` (install with `cargo install just` or `brew install just`)
# Every recipe composes standard `cargo` commands; CI uses the same underlying commands.

_default:
    @just --list --unsorted

# run zpm with arguments (e.g. `just dev --help`, `just run add react --dry-run`)
dev *args:
    cargo run -- {{args}}

run *args:
    cargo run -- {{args}}

# debug build
build:
    cargo build

# optimized release build
build-release:
    cargo build --release

# run all tests (unit + integration)
test:
    cargo test --all-features

# run unit/integration tests + doc tests
test-all:
    cargo test --all-features
    cargo test --doc

# single test by name: `just test-one detect::`
test-one filter:
    cargo test --all-features {{filter}} -- --nocapture

# static checks only (no test run)
check:
    cargo check --all-targets --all-features

# formatting
format:
    cargo fmt --all

format-check:
    cargo fmt --all -- --check

# linting (warnings denied)
lint:
    cargo clippy --all-targets --all-features -- -D warnings

# coverage (requires `cargo install cargo-llvm-cov` and nightly for branch coverage)
coverage:
    cargo +nightly llvm-cov --all-features --summary-only

coverage-html:
    cargo +nightly llvm-cov --all-features --html
    @echo "HTML coverage at target/llvm-cov/html — open target/llvm-cov/html/index.html"

coverage-open:
    cargo +nightly llvm-cov --all-features --open

coverage-lcov:
    cargo +nightly llvm-cov --all-features --lcov --output-path lcov.info
    @echo "lcov at lcov.info"

# mutation testing (requires `cargo install cargo-mutants`)
mutants:
    cargo mutants --all-features 2>&1 | head -n 100 || echo "cargo-mutants not installed — run: cargo install cargo-mutants"

mutants-list:
    cargo mutants --list --all-features || echo "cargo-mutants not installed"

# full local validation (what CI runs): fmt + lint + test + doc test + build
ci: format-check lint test-all
    cargo build
    @echo "✓ ci passed (fmt, lint, test, build)"

ci-coverage: format-check lint coverage
    cargo build
    @echo "✓ ci-coverage passed"

# dependency / security helpers (require `cargo install cargo-deny cargo-audit` if missing)
audit:
    cargo audit || echo "cargo audit not installed — run: cargo install cargo-audit"

deny:
    cargo deny check || echo "cargo deny not installed — run: cargo install cargo-deny"

outdated:
    cargo outdated || echo "cargo outdated not installed — run: cargo install cargo-outdated"

tree:
    cargo tree

# install git hooks (requires `brew install lefthook` or `go install github.com/evilmartians/lefthook@latest`)
hooks:
    lefthook install || echo "lefthook not installed — run: brew install lefthook  (or go install github.com/evilmartians/lefthook@latest)"

# bootstrap dev environment (toolchain via rust-toolchain.toml is auto-installed by rustup)
bootstrap:
    rustup show
    cargo --version
    @echo "Toolchain ready (rustfmt + clippy via rust-toolchain.toml)"
    @echo "Optional: cargo install just cargo-deny cargo-audit cargo-outdated"
    @echo "Optional hooks: brew install lefthook && lefthook install"
