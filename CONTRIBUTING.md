# Contributing to hft-optimus

> **mAI (🧠)** · Technology Lead: Murtaza Ali Imtiaz
>
> Licensed under [MIT OR Apache-2.0](LICENSE-MIT). Contributions are welcome
> under the same dual licence.

---

## Development environment

### Prerequisites

| Tool | Version | Install |
|---|---|---|
| Rust stable toolchain | ≥ 1.97.1 | `rustup update stable` |
| Rust nightly | any recent | `rustup toolchain install nightly --component rustfmt` |
| `clippy` | (with stable) | `rustup component add clippy` |
| `llvm-tools-preview` | (with stable) | `rustup component add llvm-tools-preview` |
| `cargo-llvm-cov` | latest | `cargo install cargo-llvm-cov --locked` |

### Setup

```bash
    git clone https://github.com/murtazaneuron/hft-optimus
cd hft-optimus
cp .env.example .env
# Edit .env: optionally set ANTHROPIC_API_KEY=sk-ant-...
```

---

## Workflow

### Build

```bash
cargo build           # debug
cargo build --release # optimised (use for benchmarks)
```

### Run

```bash
# Full pipeline (dry-run)
cargo run --release -- --mode full --pair SOL/USDC --amount 1.0

# Individual subsystems
cargo run --release -- --mode pev
cargo run --release -- --mode sor
cargo run --release -- --mode signer
cargo run --release -- --mode reactor
```

### Examples

```bash
cargo run --example sor_demo
cargo run --example signer_demo
cargo run --example avm_demo --release
cargo run --example jupiter_dry_run
```

### Tests (no API key required)

```bash
cargo test                   # all deterministic tests
SKIP_LLM=1 cargo test        # explicit offline mode
cargo test --test test_sor   # specific test file
```

### Live provider tests (API key required)

```bash
ANTHROPIC_API_KEY=sk-ant-... cargo test --test providers -- --ignored --test-threads=1
```

### Format, lint, docs

```bash
cargo +nightly fmt --all                             # format (nightly; rustfmt.toml uses unstable opts)
cargo +nightly fmt --all -- --check                  # CI-style check
cargo clippy --all-targets -- -D warnings            # lint (CI-strict)
cargo doc --open                                     # browse API docs
RUSTDOCFLAGS="--cfg docsrs" cargo doc                # with docsrs conditional items
```

### Coverage (cargo-llvm-cov)

```bash
# Full report: lcov.info + coverage-html/
SKIP_LLM=1 cargo llvm-cov --workspace \
  --lcov --output-path lcov.info \
  --ignore-filename-regex 'tests/'
SKIP_LLM=1 cargo llvm-cov report --html --output-dir coverage-html
open coverage-html/index.html          # browse line-by-line HTML report

# Quick terminal summary
SKIP_LLM=1 cargo llvm-cov --workspace --summary-only

# Clean stale instrumentation artefacts after a cancelled run
cargo llvm-cov clean --workspace
```

---

## Code style

- **Edition**: Rust 2024
- **MSRV**: 1.97.1 (enforced by `rust-version` in `Cargo.toml` and `msrv` in
  `.clippy.toml`)
- **Max line width**: 100 characters (enforced by `rustfmt.toml`)
- **Imports**: use the multi-line form for `rig` imports; both
  `rig::client::CompletionClient` and `rig::client::ProviderClient` must be imported
  when calling `.agent()` on any rig-core 0.36+ Anthropic client
- **Doc comments**: `//!` for module-level docs; `///` for items; never `///` on
  macro invocation sites (triggers `unused_doc_comments`)
- **Error handling**: always `anyhow::Result`; propagate with `?`; no `unwrap` in
  library code

---

## Adding a new venue adapter (SOR)

1. Add the SDK crate to `Cargo.toml`
2. Implement the `query_*` function in the relevant stub file (`src/sor/raydium.rs`, etc.)
3. Make it `pub` and re-export from `src/sor/mod.rs`
4. Remove the stub query in `src/sor/router.rs` and import the real one
5. Add a test in `tests/test_sor.rs` and optionally a live test in
   `tests/providers/anthropic.rs`

---

## CI

The CI pipeline (`.github/workflows/ci.yml`) runs on every push and pull request
to `main`. It has two parallel jobs:

### `fmt` job — nightly rustfmt

`rustfmt.toml` uses nightly-only options (`imports_granularity`, `group_imports`,
`wrap_comments`, etc.) enabled by `unstable_features = true`. The format check runs
under `dtolnay/rust-toolchain@nightly` in its own job so the nightly toolchain never
interferes with the matrix job.

| Step | Command |
|---|---|
| Format check | `cargo fmt --all -- --check` (nightly rustfmt) |

### `ci` job — stable + MSRV matrix (`stable`, `1.97.1`)

| Step | What it enforces |
|---|---|
| Clippy | `cargo clippy --all-targets -- -D warnings` — zero warnings (stable only) |
| Build | `cargo build --release` — release binary compiles |
| Tests | `SKIP_LLM=1 cargo test --workspace` — all deterministic tests pass |
| Coverage | `cargo llvm-cov --workspace --lcov …` + HTML report (stable only) |
| HTML artifact | `coverage-html/` uploaded, retained 30 days |
| Docs | `RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo doc --no-deps` (stable only) |
| MSRV | Full build + test on Rust 1.97.1 |

**Coverage toolchain:** `llvm-tools-preview` is installed as a toolchain component;
`cargo-llvm-cov` binary is installed via `taiki-e/install-action@v2`.
The `target/` directory is intentionally **not** cached — at ~3 GB it causes
step-timeout cancellations; the registry cache (~100 MB) is sufficient.

---

## Publishing

```bash
# Verify the published package looks correct (no upload)
cargo publish --dry-run

# Publish to crates.io
cargo publish
```

Set a crates.io API token via `cargo login` or `CARGO_REGISTRY_TOKEN`.
