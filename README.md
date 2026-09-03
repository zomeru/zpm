# zpm — Package Manager Abstraction CLI

> **One CLI. Detect the package manager. Translate the intent. Run the right command.**

zpm is a fast, polished, cross-platform Rust CLI that abstracts over the modern JavaScript/TypeScript package-manager ecosystem.

```bash
zpm install
zpm add react
zpm add typescript --dev
zpm run dev
zpm exec vite --host 0.0.0.0
zpm update
zpm remove react
zpm dedupe
```

zpm detects your project's package manager (npm, pnpm, Yarn Classic, Yarn Berry, Bun, Deno, Aube, Nub, Rush/pnpm) and translates the generic command to the correct native invocation:

```bash
zpm add react
# → npm install react
# → pnpm add react
# → yarn add react
# → bun add react
# → deno add npm:react  (handled as deno add)
# → aube add react
```

## Why zpm?

- **Zero Node.js requirement** — native binary, instant startup
- **Robust detection** — lockfiles, `packageManager` field, `devEngines`, `deno.json`, workspace files
- **Reliable argument forwarding** — correct handling of `--` for npm and other managers
- **Great terminal UX** — colors, interactive pickers, dry-run previews, useful errors
- **Workspace-aware** — nested packages resolve to workspace root, `--root` targets root explicitly
- **CI-friendly** — respects `NO_COLOR`, non-interactive terminals, and provides clean output

## Installation

### Cargo

```bash
cargo install zpm
```

### From source

```bash
git clone https://github.com/zomeru/zpm
cd zpm
cargo build --release
# binary at target/release/zpm
```

### Prebuilt binaries

Prebuilt binaries for `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `aarch64-apple-darwin`, `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc` are attached to GitHub Releases.

### Homebrew / Scoop / Winget

Future distribution will include:

```bash
brew install zpm
scoop install zpm
winget install zpm
```

## Supported Package Managers

| Manager | Lockfiles | Global | Frozen | Dedupe | Execute |
|---------|-----------|--------|--------|--------|---------|
| npm | `package-lock.json`, `npm-shrinkwrap.json` | ✓ (`-g`) | `npm ci` | `npm dedupe` | `npx` |
| pnpm | `pnpm-lock.yaml`, `pnpm-workspace.yaml` | ✓ | `pnpm i --frozen-lockfile` | `pnpm dedupe` | `pnpm dlx` / `pnpm exec` |
| pnpm@6 | `pnpm-lock.yaml` + `packageManager: pnpm@6` | ✓ | same | ✓ | same |
| pnpm via Rush | `rush.json`, `common/config/rush/pnpm-lock.yaml` | ✓ | same via `rush-pnpm` | ✓ | same via `rush-pnpm` |
| Yarn Classic | `yarn.lock` + `packageManager: yarn@1` | `yarn global add` | `--frozen-lockfile` | — | `npx` / `yarn exec --` |
| Yarn Berry | `yarn.lock` + `packageManager: yarn@2+` or `.yarnrc.yml` | via `npm i -g` | `--immutable` | `yarn dedupe` | `yarn dlx` / `yarn exec` |
| Bun | `bun.lock`, `bun.lockb` | ✓ | `--frozen-lockfile` | — | `bun x` |
| Deno | `deno.lock`, `deno.json` | `deno install -g` | `--frozen` | — | `deno x` / `deno task --eval` |
| Aube | `aube-lock.yaml`, `aube-workspace.yaml` | ✓ | `--frozen-lockfile` | ✓ | `aube dlx` / `aube exec` |
| Nub | `nub.lock` | ✓ | `--frozen-lockfile` | ✓ | `nubx` / `nub exec` |

Adding a new manager is straightforward: implement its capability table in `src/package_manager/mod.rs` — no scattered `if/else` blocks.

## Detection

zpm walks from the current directory up to the filesystem root and checks, in order at each directory:

1. **`rush.json`** → `pnpm-rush`
2. **Lockfiles** (`pnpm-lock.yaml`, `yarn.lock`, `bun.lockb`, etc.) — if a `package.json` with a `packageManager` field exists in the same directory, that takes precedence
3. **`packageManager` / `devEngines.packageManager`** in `package.json`:

   ```json
   { "packageManager": "pnpm@9.1.0" }
   { "packageManager": "yarn@3.2.0" }
   { "devEngines": { "packageManager": { "name": "pnpm", "version": "9.0.0" } } }
   ```

   - `pnpm@<7` → `pnpm@6` (legacy `pnpm i --frozen-lockfile` behavior with different run semantics)
   - `yarn@>1` → `yarn@berry`
4. **Install metadata** (`node_modules/.pnpm`, `node_modules/.yarn-state.yml`, `.pnp.cjs`, etc.)
5. **`deno.json` / `deno.jsonc`** (checked early for the target directory, then again as ancestor fallback)

If detection is ambiguous (multiple lockfiles in the same directory) zpm uses deterministic precedence mirroring `package-manager-detector` and will surface a prompt when running in an interactive terminal.

**Deno shortcut:** `deno.json` in the current directory short-circuits to `deno` immediately, matching `ni`.

### Overrides

```bash
zpm --pm pnpm add react      # force pnpm
zpm --pm bun install
ZPM_PM=pnpm zpm install      # env override
```

## Commands

All commands support `--dry-run` (show without executing) and `--verbose` (show detection + command).

```bash
zpm --help
zpm add --help
```

### `zpm install` / `zpm i`

Install dependencies.

```bash
zpm install
zpm install --frozen
zpm install --frozen-if-present
zpm install --production         # npm → --omit=dev
zpm i -P                        # alias
```

### `zpm add` / `zpm a`

Add dependencies. Flags mirror common PM conventions:

```bash
zpm add react react-dom
zpm add typescript --dev         # -D
zpm add typescript -D            # bun → -d automatically
zpm add eslint --peer
zpm add eslint --exact -E
zpm add react --global -g        # global
```

### `zpm remove` / `zpm rm`

```bash
zpm remove react
zpm rm react lodash
zpm remove --global eslint
zpm remove                       # interactive multi-select (TTY)
zpm remove -i                   # explicit interactive
```

### `zpm update` / `zpm up`

```bash
zpm update
zpm update react
zpm update -i --latest          # interactive latest
zpm up -i
```

Note: `npm` has no `upgrade-interactive`; zpm falls back to `npm update`.

### `zpm run` / `zpm r`

Run scripts. Handles npm's `--` insertion and workspace flags:

```bash
zpm run dev
zpm run dev --port 3000          # npm → npm run dev -- --port 3000
zpm run build --watch -o
zpm run dev -- --port 3000       # explicit --
zpm run -w packages/foo test     # before script → -w=packages/foo test
zpm run test -w packages/foo     # after script  → test -- -w=packages/foo
zpm run --if-present test
zpm run                          # interactive fuzzy picker
zpm run -p dev                   # strip -p monorepo prefix
```

### `zpm exec` / `zpm x` / `zpm dlx`

```bash
zpm exec vite
zpm x vite --host 0.0.0.0        # pnpm → pnpm dlx vite ...
zpm exec --local esbuild        # pnpm → pnpm exec, yarn classic → yarn exec -- --version
zpm x --local vitest
```

### `zpm dedupe`

```bash
zpm dedupe
zpm dedupe --check -c            # npm → --dry-run, pnpm/aube → --check
```

Unsupported managers (e.g., `bun dedupe`, `deno dedupe`) return a clear error: `× dedupe not supported for bun`.

### `zpm clean` / `zpm ci`

Clean (frozen) install — alias for `--frozen`:

```bash
zpm clean
# npm → npm ci
# pnpm → pnpm i --frozen-lockfile
# yarn classic → yarn install --frozen-lockfile
# yarn berry → yarn install --immutable
# bun → bun install --frozen-lockfile
# deno → deno install --frozen
```

### `zpm agent`

Print detected agent name (for scripting):

```bash
zpm agent
# npm
```

### Bare invocation (ni compatibility)

```bash
zpm                # → install
zpm react          # → add react
zpm react lodash   # → add react lodash
zpm --frozen       # → frozen install
zpm -g eslint      # → global add
```

## Argument Forwarding

zpm preserves arguments as separate argv entries via `std::process::Command` — no shell-string concatenation.

- `zpm run dev -- --port 3000` and `zpm run dev --port 3000` behave correctly per manager (npm requires `--` before forwarded args, others do not)
- `zpm add react react-dom` forwards both as packages
- `zpm exec vite --host 0.0.0.0` forwards correctly; `yarn exec` uses `--` when needed

## Workspace Support

zpm detects workspace roots via:

- `pnpm-workspace.yaml`
- `.yarnrc.yml`
- `rush.json`
- `deno.json` / `deno.jsonc`
- `package.json` `workspaces` field

From a nested package, commands resolve against the workspace root. `--root` explicitly targets the root:

```bash
zpm --root install
```

Package script discovery also supports monorepo package selection via interactive prompts when multiple `package.json` files are found.

## Catalogs

zpm includes architecture for modern dependency catalogs:

- pnpm catalogs (`pnpm-workspace.yaml` `catalog` / `catalogs`)
- Yarn Berry catalogs (`.yarnrc.yml`)
- Bun catalogs (`package.json` `workspaces.catalog` / top-level `catalog` / `catalogs`)

Current implementation provides provider detection and `catalog:` reference generation; full read-modify-write of workspace catalog files (with proper YAML/JSON serialization and preservation) is scaffolded for the next milestone. The architecture avoids brittle string replacement.

To disable catalog mode: `ZPM_CATALOG=false` or `NI_CATALOG=false`.

## Configuration

zpm reads (first existing wins):

- `~/.config/zpm/config.toml`
- `~/.nirc` (ini, for `ni` compatibility)

Example `~/.config/zpm/config.toml`:

```toml
default_manager = "pnpm"
global_manager = "pnpm"
interactive = true
color = "auto"      # auto | always | never
auto_install = false # if true, auto-install missing PM via Corepack/npm
catalog = true
```

### Environment Variables

| Variable | Description |
|----------|-------------|
| `ZPM_DEFAULT_MANAGER` | Default manager when none detected (also `NI_DEFAULT_AGENT`) |
| `ZPM_GLOBAL_MANAGER` | Manager for `-g` operations (also `NI_GLOBAL_AGENT`) |
| `ZPM_CONFIG` | Custom config file path (also `NI_CONFIG_FILE`; `"false"` disables) |
| `ZPM_PM` / `zpm --pm` | Per-invocation override |
| `ZPM_AUTO_INSTALL` | `true` to auto-install missing PM |
| `ZPM_NO_INTERACTIVE` | `true` to disable prompts |
| `ZPM_CATALOG` | `false` to disable catalog mode (`NI_CATALOG`) |
| `ZPM_DRY_RUN` | `true` acts like `--dry-run` |
| `ZPM_VERBOSE` | `true` acts like `--verbose` |
| `NO_COLOR` | Disable colors (standard) |
| `CI` | In CI, missing PM causes immediate exit; default agent falls back to `npm` |

Precedence: CLI flags > environment > config file > defaults.

## Dry Run & Verbose

```bash
zpm add react --dry-run
# Detected package manager: pnpm
# Command: pnpm add react

zpm --verbose add react
# Detected: pnpm
# Command: pnpm add react
# (then executes)
```

Both flags are available globally:

```bash
zpm --dry-run --pm pnpm add react
zpm --verbose run dev
```

## Auto-Install

When zpm detects a manager that is not installed:

- In CI: exits with helpful install hint
- Interactively: prompts to install globally via `npm i -g <manager>` (requires confirmation)
- With `ZPM_AUTO_INSTALL=true`: installs automatically
- Prefer Corepack-managed managers where applicable (`corepack enable` is recommended for pnpm/Yarn)

Never installs silently unless explicitly enabled.

## Shell Completion

```bash
# Bash
zpm completion --bash >> ~/.bashrc
# Zsh
zpm completion --zsh >> ~/.zshrc
# Fish
zpm completion --fish >> ~/.config/fish/completions/zpm.fish
```

Block is also emitted for `nr`-compatible tooling via `zpm completion`.

## Comparison with `ni`

zpm recreates `ni`'s core purpose but with a deliberate, idiomatic Rust architecture:

| Aspect | `ni` | zpm |
|--------|------|-----|
| Language | TypeScript / Node.js | Rust (native) |
| Startup | Node.js VM | < 5ms native |
| CLI style | Separate binaries (`ni`, `nr`, `nlx`, …) | Unified `zpm` with subcommands + `ni`-compatible bare mode |
| Detection | `package-manager-detector` (JS) | Native Rust port with same precedence |
| Config | `~/.nirc` (ini) | `~/.config/zpm/config.toml` + `~/.nirc` compat + env |
| Catalogs | Full pnpm/Yarn/Bun support (recent) | Scaffolded provider model (full YAML/JSON impl next milestone) |
| Interactive | `prompts` + `fzf` | `dialoguer` fuzzy/select/multi |
| Process | `tinyexec` (`sh -c` on some paths) | `std::process::Command` argv-preserving, no shell |
| Global state | Module-level config | Explicit passing, no global mut |
| Windows | Supported via Node | First-class Rust `PathBuf` handling |

**Behavioral compatibility:** `zpm` aims for `zpm result ≈ ni result` for all core translations. Intentional differences are documented:

- `zpm` is a single binary; `ni`, `nr`, `nlx`, etc. map to `zpm install`/`add`, `zpm run`, `zpm exec`
- `yarn@berry` global delegates to `npm i -g` (Berry has no global) — same as `ni` / `package-manager-detector`
- `pnpm@6` is distinguished via `packageManager: pnpm@6` for correct `run` dash semantics
- `--dry-run` is explicit (ni uses `?` token for debug)
- `zpm clean` is an explicit alias for frozen install

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for the full workflow. Quick start:

```bash
rustup show                  # installs toolchain from rust-toolchain.toml (stable + rustfmt + clippy)

# formatting / lint / test / build (same commands CI runs)
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --doc
cargo build --release
./target/release/zpm --help

# or with `just` (cargo install just)
just format-check
just lint
just test-all
just ci          # full local CI validation
just dev --help  # run zpm with args: cargo run -- --help
```

### Project Structure

```
src/
  main.rs
  lib.rs
  cli/          # Clap definitions + high-level resolvers (install/add/run/exec…)
  config/       # TOML/ini + env precedence
  detection/    # Lockfile/packageManager/devEngines/deno.json walk
  package_manager/ # Agent enum + central COMMANDS capability table + resolve_command
  process/      # std::process::Command execution (argv-preserving)
  ui/           # Colors, prompts, spinners (respects NO_COLOR)
  workspace/    # Workspace root + package.json helpers
  catalog/      # Provider model for pnpm/Yarn/Bun catalogs
  error.rs
```

## License

MIT — see [LICENSE](./LICENSE).