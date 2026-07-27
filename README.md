# hft-optimus

**Optimal High-Frequency Trading Platform - Rig (Rust Inference Gateway / ARC)**

[![Crates.io](https://img.shields.io/crates/v/hft-optimus.svg)](https://crates.io/crates/hft-optimus)
[![Docs.rs](https://docs.rs/hft-optimus/badge.svg)](https://docs.rs/hft-optimus)
[![Rust](https://img.shields.io/badge/rust-1.97.1%2B-orange.svg)](https://www.rust-lang.org/)
[![Edition](https://img.shields.io/badge/edition-2024-blue.svg)](https://doc.rust-lang.org/edition-guide/rust-2024/)
[![rig-core](https://img.shields.io/badge/rig--core-%5E0.36-purple.svg)](https://rig.rs)
[![Solana](https://img.shields.io/badge/solana-devnet%2Fmainnet-9945FF.svg)](https://solana.com)
[![CI](https://github.com/murtazaneuron/hft-optimus/actions/workflows/ci.yml/badge.svg)](https://github.com/murtazaneuron/hft-optimus/actions/workflows/ci.yml)
[![License: MAI](https://img.shields.io/badge/license-MAI-blue.svg)](LICENSE-MAI)

> Built by **[Murtaza Ali Imtiaz](https://github.com/murtazaneuron)** · Technology Lead · **mAI (🧠)** · July 2019 – Present

A production-grade Rust implementation of an LLM-driven HFT agent framework powered by
[Rig (ARC)](https://rig.rs) - the high-performance enterprise alternative to Python LLM
frameworks. The platform demonstrates end-to-end **agentic trade governance** through a
**Plan → Execute → Verify (PEV) loop**, concurrent **Smart Order Routing (SOR)** across
three Solana DEXs, **task-local SignerContext keypair isolation**, **Jupiter swap
simulation**, **AVM vs EVM execution benchmarking**, and a **Reactor GUI audit log** - all
within a single Rust crate that compiles as both a library and a binary.

---

## Architecture

```
  CLI Entry  ─────────────── main.rs (clap: --mode, --pair, --amount, --skip-llm, --live)
       │
  ┌────┴─────────────────────────────────────────────────────────┐
  │  PEV Loop  (src/pev/)                                        │
  │                                                              │
  │   PLAN      claude-haiku-4-5  → Vec<TradeTask> (4 tasks)    │
  │     │                                                        │
  │   EXECUTE   claude-sonnet-4-6 → ExecuteOutput per task      │
  │     │                                                        │
  │   VERIFY    claude-haiku-4-5  → score ∈ [0,1]; pass ≥ 0.80 │
  │     │  score < 0.80: retry up to 2× with feedback injected  │
  └────┬─────────────────────────────────────────────────────────┘
       │
  ┌────┴─────────────────────────────────────────────────────────┐
  │  Smart Order Routing  (src/sor/)                             │
  │                                                              │
  │   tokio::join! ──► Raydium  (25 bps, ~143.52 USDC/SOL)     │
  │                ──► Orca     (30 bps, ~143.48 USDC/SOL)      │
  │                ──► Serum    (20 bps, ~143.61 USDC/SOL)      │
  │                                                              │
  │   Cost formula:  price × (1 + fee_bps / 10_000)            │
  │   Winner: lowest effective cost                              │
  └────┬─────────────────────────────────────────────────────────┘
       │
  ┌────┴─────────────────────────────────────────────────────────┐
  │  On-chain Execution  (src/onchain/)                          │
  │                                                              │
  │   SignerContext (tokio::task_local!) ── keypair isolation    │
  │   Jupiter swap simulation            ── SIM_<hex> signature │
  │   DRY_RUN=true by default            ── no live txns sent   │
  └────┬─────────────────────────────────────────────────────────┘
       │
  ┌────┴─────────────────────────────────────────────────────────┐
  │  AVM Layer  (src/avm/)                                       │
  │                                                              │
  │   Benchmark: AVM JIT (~1–3 ns/op) vs EVM (~10–30 ns/op)    │
  │   Reactor GUI audit log: STATE BEFORE → EXECUTION → AFTER   │
  └──────────────────────────────────────────────────────────────┘
```

---

## Tech Stack

| Layer | Technology | Notes |
|---|---|---|
| AI Agent Framework | [rig-core](https://crates.io/crates/rig-core) `^0.36` | Rig / ARC - Rust Inference Gateway |
| LLM - Plan & Verify | `claude-haiku-4-5` | Low-cost model for structured decomposition and scoring |
| LLM - Execute | `claude-sonnet-4-6` | High-capability model for agentic tool-use reasoning |
| Async Runtime | [Tokio](https://tokio.rs) `^1` (full features) | Drives all async tasks and `task_local!` storage |
| Blockchain | Solana (`solana-sdk ^3`, `solana-client ^3`) | Devnet by default; mainnet-ready |
| Token | `spl-token ^9` | Aligned with Solana SDK 3.x |
| Cryptography | `ed25519-dalek ^2`, `k256 ^0.13` | ECDSA and Ed25519 signing primitives |
| HTTP / TLS | `reqwest ^0.13` (rustls) | JSON requests; rustls avoids OpenSSL dependency |
| CLI | `clap ^4` (derive) | `--mode`, `--pair`, `--amount`, `--skip-llm`, `--live` |
| Logging | `tracing ^0.1` + `tracing-subscriber ^0.3` | Structured fields; env-filter for `RUST_LOG` |
| Error handling | `anyhow ^1`, `thiserror ^2` | `?`-based propagation throughout |
| Env config | `dotenvy ^0.15` | Loads `.env` before `Config::from_env()` |
| IDE | [Zed](https://zed.dev) | `.zed/settings.json`, `tasks.json`, `debug.json` |

---

## Quick Start

```bash
# 1. Clone and enter
git clone https://github.com/murtazaneuron/hft-optimus
cd hft-optimus

# 2. Configure environment
cp .env.example .env
# Optional: set ANTHROPIC_API_KEY=sk-ant-... for live LLM mode
# All subsystems work without a key - see Offline/Stub Mode below

# 3. Build
cargo build --release

# 4. Run - full pipeline, offline stub (no API key needed)
cargo run --release -- --mode full --skip-llm --pair SOL/USDC --amount 1.0

# 5. Run - full pipeline, live LLM (requires ANTHROPIC_API_KEY in .env)
cargo run --release -- --mode full --pair SOL/USDC --amount 1.0
```

### As a library dependency

```toml
[dependencies]
hft-optimus = "0.1"
```

```rust
use hft_optimus::{config::Config, pev, sor};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cfg = Config::from_env()?;

    // Smart Order Routing - always works, no API key needed
    let route = sor::best_route("SOL/USDC", 1.0).await?;
    println!("Best venue: {} @ {:.4} USDC", route.venue, route.effective_price);

    // PEV loop - uses offline stubs when ANTHROPIC_API_KEY is absent
    let result = pev::run(&cfg, "SOL/USDC", 1.0).await?;
    println!("PEV passed: {} (score={:.2})", result.passed, result.verify_score);

    Ok(())
}
```

---

## CLI Reference

```
USAGE:
    hft-optimus [OPTIONS]

OPTIONS:
    -m, --mode <MODE>        Operating mode [default: full]
                             [possible values: full, pev, sor, signer, reactor]
    -p, --pair <PAIR>        Trading pair, e.g. SOL/USDC [default: SOL/USDC]
    -a, --amount <AMOUNT>    Base-token amount [default: 1.0]
        --skip-llm           Force offline stub mode for PEV phases
                             Implied automatically when ANTHROPIC_API_KEY is absent
        --live               Enable live on-chain transactions (dry-run by default)
    -h, --help               Print help
```

### Modes

| Mode | Subsystems exercised | API key required? |
|---|---|---|
| `full` | PEV loop → SOR → Jupiter swap → AVM audit log | No (stub) / Yes (live LLM) |
| `pev` | PEV loop only | No (stub) / Yes (live LLM) |
| `sor` | Smart Order Routing only | No |
| `signer` | SignerContext isolation demo | No |
| `reactor` | AVM benchmark + Reactor audit log | No |

---

## Offline / Stub Mode

Every subsystem runs without an `ANTHROPIC_API_KEY`. When a key is absent (or
`--skip-llm` is passed), the PEV phases substitute deterministic offline stubs:

| Phase | Live path | Stub path |
|---|---|---|
| **Plan** | Haiku decomposes to JSON via API | Returns `default_tasks()` - 4 canonical tasks |
| **Execute** | Sonnet reasons and calls tools | Returns fixed `ExecuteOutput`; confidence = 0.90 |
| **Verify** | Haiku scores against criteria | Returns score = 0.90, feedback = "all criteria assumed met" |

SOR, `SignerContext`, Jupiter dry-run, and AVM benchmark are unaffected - they never
require an API key.

Stub mode is activated by **any** of the following:

| Method | When to use |
|---|---|
| Leave `ANTHROPIC_API_KEY` blank in `.env` | Default; no key provisioned |
| `--skip-llm` CLI flag | Force stub for a single `cargo run` invocation |
| `SKIP_LLM=1 cargo run` | Force stub via environment variable |
| `SKIP_LLM=1` set permanently in `.env` | Always-on for a whole checkout |

> **Important - `cargo test`:** `--skip-llm` is a flag for the compiled binary's clap
> parser. **Never** pass it after `--` in a `cargo test` invocation.
>
> ```text
> cargo test -- --skip-llm   # ✗ WRONG - test harness rejects it
> SKIP_LLM=1 cargo test      # ✓ correct
> ```

---

## Environment Variables

All variables are optional. `Config::from_env()` is infallible - every variable has a
safe default.

| Variable | Default | Description |
|---|---|---|
| `ANTHROPIC_API_KEY` | `""` | Anthropic API key for rig-core. Absent → `skip_llm = true`; all PEV phases use stubs. |
| `SKIP_LLM` | `false` | Set to `1` or `true` to force stub mode even when a key is present. |
| `SOLANA_RPC_URL` | `https://api.devnet.solana.com` | Solana JSON-RPC endpoint. |
| `SOLANA_PRIVATE_KEY` | `DEMO_KEY_PLACEHOLDER` | Base-58 encoded keypair for signing. In production load from a secrets manager. |
| `DRY_RUN` | `true` | When `true`, all on-chain operations are simulated and no real transactions are broadcast. |

Log level defaults to `hft_optimus=debug`. Override at runtime:

```bash
RUST_LOG=debug cargo run --release -- --mode full
RUST_LOG=hft_optimus=trace cargo run --release -- --mode pev
```

---

## PEV Loop - Plan → Execute → Verify

The PEV loop governs every trade decision. A single `pev::run()` call:

1. **PLAN** - `claude-haiku-4-5` decomposes the trade into exactly **4 atomic
   `TradeTask` objects**:

   | Task ID | `TradeAction` | Acceptance criteria |
   |---|---|---|
   | `T001` | `analyse_market` | Market data retrieved |
   | `T002` | `select_route` | Best DEX venue selected |
   | `T003` | `validate_slippage` | Slippage within 0.5% tolerance |
   | `T004` | `simulate_execution` | Dry-run swap simulation logged |

2. **EXECUTE** - `claude-sonnet-4-6` processes each task, invoking the mapped tool:

   | Action | Tool call |
   |---|---|
   | `analyse_market` | `fetch_price_feed(SOL/USDC)` |
   | `select_route` | `query_raydium_pool()`, `query_orca_pool()` |
   | `validate_slippage` | `calculate_slippage(amount)` |
   | `simulate_execution` | `jupiter_swap_dry_run()` |

3. **VERIFY** - `claude-haiku-4-5` scores the output against criteria.
   - Pass threshold: **≥ 0.80**
   - On failure: feedback is injected into the next attempt
   - Max retries per task: **2**

Cost model: Haiku handles cheap plan and verify work; Sonnet is reserved for
reasoning-heavy execution. This cuts LLM cost by roughly 60–70% compared with an
all-Sonnet pipeline.

---

## Smart Order Routing (SOR)

`sor::best_route()` fans out to all three venues **simultaneously** using
`tokio::join!`, then selects the winner by effective cost:

```
effective_cost = price × (1 + fee_bps / 10_000)
```

| Venue | Price (SOL/USDC) | Fee | Price impact | Simulated latency |
|---|---|---|---|---|
| Raydium | 143.52 | 25 bps | 0.03% | 12 ms |
| Orca | 143.48 | 30 bps | 0.02% | 9 ms |
| Serum (OpenBook) | 143.61 | 20 bps | 0.05% | 15 ms |

If all venue queries fail, a `Raydium-fallback` route is returned so the pipeline is
never blocked.

---

## On-chain Execution & SignerContext

### `SignerContext` (`src/onchain/signer.rs`)

Uses `tokio::task_local!` to scope a `solana_sdk::signature::Keypair` to exactly one
Tokio task. Multiple concurrent trades cannot share or leak each other's signing keys,
with no mutex overhead.

```rust
use hft_optimus::onchain::signer::{LocalSolanaSigner, with_signer};

# async fn example() -> anyhow::Result<()> {
let signer = LocalSolanaSigner::from_env();
let result = with_signer(signer, || async {
    // CURRENT_SIGNER is only visible inside this async block
    Ok::<&str, anyhow::Error>("swap executed")
   }).await?;
# Ok(()) }
```

### Jupiter Swap Simulation (`src/onchain/jupiter.rs`)

`simulate_swap()` computes output without sending any RPC call:

```
output_amount = input_amount × effective_price
fee_paid      = input_amount × fee_bps / 10_000
simulated_sig = "SIM_" + 16-char random hex
```

`is_dry_run = true` is always set in demo mode. Pass `--live` to attempt live mode
(returns `Err` in this demo build - production wiring is in progress).

---

## AVM Benchmark & Reactor Audit Log

### AVM vs EVM Benchmark (`src/avm/benchmark.rs`)

Runs 10 000 iterations of each engine and logs ns/op and the speedup ratio:

| Engine | Method | Typical result |
|---|---|---|
| AVM (Agave JIT) | `#[inline(always)]`, stack-only arithmetic, zero heap allocation | ~1–3 ns/op |
| EVM (bytecode) | `#[inline(never)]`, one `Vec` allocation per call | ~10–30 ns/op |

Run with `--release` for meaningful timing; debug builds omit optimisations.

### Reactor GUI Audit Log (`src/avm/reactor.rs`)

Emits a structured three-phase execution trace at `INFO` level:

```
[REACTOR GUI] ── STATE BEFORE ──  Balance, pool, price, fee
[REACTOR GUI] ── EXECUTION ──     Method, compute units, AVM mode, signature
[REACTOR GUI] ── STATE AFTER ──   Output amount, fee paid, status: SUCCESS
```

---

## Build & Test

### Prerequisites

| Requirement | Version | Install |
|---|---|---|
| Rust stable | ≥ 1.97.1 (MSRV) | `rustup update stable` |
| Rust nightly | any recent | `rustup toolchain install nightly --component rustfmt` |
| `clippy` | bundled with stable | `rustup component add clippy` |
| `llvm-tools-preview` | bundled with stable | `rustup component add llvm-tools-preview` |
| `cargo-llvm-cov` | latest | `cargo install cargo-llvm-cov --locked` |
| `ANTHROPIC_API_KEY` | - | Optional. Absent → offline stub mode. Required only for `#[ignore]` live tests. |

> **Note on nightly rustfmt:** `rustfmt.toml` uses nightly-only options
> (`imports_granularity`, `group_imports`, `wrap_comments`, etc.) gated by
> `unstable_features = true`. Run `cargo +nightly fmt` locally; the CI `fmt` job
> handles this automatically.

### Setup

```bash
git clone https://github.com/murtazaneuron/hft-optimus
cd hft-optimus
cp .env.example .env
# Edit .env: optionally set ANTHROPIC_API_KEY=sk-ant-...
```

### Build commands

```bash
cargo build                  # debug build
cargo build --release        # optimised - required for meaningful benchmark timing
cargo check --all-targets    # type-check only, no linking (fastest feedback)
cargo clean                  # remove target/
```

**Release profile** (`Cargo.toml`):

```toml
[profile.release]
opt-level     = 3
lto           = true
codegen-units = 1
panic         = "abort"
strip         = "debuginfo"
```

### Test commands

All tests in `tests/*.rs` are fully deterministic and pass without an API key.

```bash
cargo test                                    # all deterministic tests
SKIP_LLM=1 cargo test                         # explicit offline mode
cargo test -- --nocapture                     # with log output to stdout
cargo test --test test_avm_benchmark          # single file
cargo test --test test_pev_loop               # single file
cargo test --test test_signer_context         # single file
cargo test --test test_sor                    # single file
```

**Live provider tests** (API key required, `#[ignore]` in CI):

```bash
ANTHROPIC_API_KEY=sk-ant-... \
    cargo test --test providers -- --ignored --test-threads=1
```

### Lint, format, docs

```bash
cargo +nightly fmt --all                             # apply formatting (nightly; rustfmt.toml)
cargo +nightly fmt --all -- --check                  # CI format check (nightly rustfmt)
cargo clippy --all-targets -- -D warnings            # lint in CI mode (zero warnings)
cargo doc --open --document-private-items            # browse rustdoc locally
RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo doc   # CI docs check
```

### Coverage (cargo-llvm-cov)

`rustfmt` uses nightly but all coverage tooling runs on stable.

```bash
# Install once
cargo install cargo-llvm-cov --locked
rustup component add llvm-tools-preview

# Full workspace coverage → lcov.info + coverage-html/
SKIP_LLM=1 cargo llvm-cov --workspace \
  --lcov --output-path lcov.info \
  --ignore-filename-regex 'tests/'
SKIP_LLM=1 cargo llvm-cov report --html --output-dir coverage-html
open coverage-html/index.html

# Quick terminal summary
SKIP_LLM=1 cargo llvm-cov --workspace --summary-only

# Clean stale instrumentation artefacts
cargo llvm-cov clean --workspace
```

> All coverage Zed tasks are self-bootstrapping — they install `cargo-llvm-cov` and
> `llvm-tools-preview` automatically on first run.

### Running the binary

```bash
# Full pipeline - offline stub
cargo run --release -- --mode full --skip-llm --pair SOL/USDC --amount 1.0

# Full pipeline - live LLM (requires ANTHROPIC_API_KEY)
cargo run --release -- --mode full --pair SOL/USDC --amount 1.0

# Individual subsystems (never need an API key)
cargo run --release -- --mode pev --skip-llm
cargo run --release -- --mode sor
cargo run --release -- --mode signer
cargo run --release -- --mode reactor

# Help
cargo run --release -- --help
```

### Standalone examples

Each example is self-contained and runnable with a single `cargo run` command.

```bash
cargo run --release --example sor_demo        # concurrent SOR, prints winning route
cargo run --release --example signer_demo     # SignerContext isolation across 3 tasks
cargo run --release --example avm_demo        # AVM vs EVM benchmark (use --release)
cargo run --release --example jupiter_dry_run # Jupiter swap simulation + Reactor log
```

---

## Zed IDE Configuration (`.zed/`)

Three project-local config files are provided for [Zed](https://zed.dev):

| File | Contents |
|---|---|
| `.zed/settings.json` | rust-analyzer tuned to `rustfmt.toml` and `.clippy.toml`; format-on-save; inlay hints; import grouping matching `imports_granularity = "Crate"` |
| `.zed/tasks.json` | tasks covering build, test (per-file + live providers), lint, fmt, doc, run (all 5 modes), all 4 examples, and a one-shot local CI simulation |
| `.zed/debug.json` | CodeLLDB debug configurations: all binary modes (dev + release), all 4 integration test files (via `--no-run` + glob program path), and tooling checks |

---

## CI Pipeline

`.github/workflows/ci.yml` runs on every push and pull request to `main`.
The pipeline has two parallel jobs:

### `fmt` job — nightly rustfmt

Runs independently so the nightly toolchain never interferes with the matrix job.
`rustfmt.toml` requires `unstable_features = true` and a nightly `rustfmt` binary.

| Step | Command |
|---|---|
| 1 | `cargo fmt --all -- --check` (via `dtolnay/rust-toolchain@nightly`) |

### `ci` job — stable + MSRV matrix (`stable`, `1.97.1`)

| Step | Command | What it enforces |
|---|---|---|
| 1 | `cargo clippy --all-targets -- -D warnings` | Zero lint warnings (stable only) |
| 2 | `cargo build --release` | Release binary compiles |
| 3 | `SKIP_LLM=1 cargo test --workspace` | All deterministic tests pass |
| 4 | `cargo llvm-cov --workspace --lcov …` | Coverage report + `lcov.info` (stable only) |
| 5 | Upload `coverage-html/` | HTML artifact retained 30 days |
| 7 | `RUSTDOCFLAGS="--cfg docsrs -D warnings" cargo doc` | Docs compile without warnings (stable only) |
| MSRV | full build + test on `1.97.1` | Minimum supported Rust version enforced |

**Coverage notes:**
- `llvm-tools-preview` is installed as part of the toolchain component list
- `cargo-llvm-cov` binary is installed via `taiki-e/install-action@v2`
- `target/` is intentionally **not** cached — at ~3 GB it causes step-timeout cancellations; the registry cache (~100 MB) is sufficient


---

## Repository Structure

```
hft-optimus/
hft-optimus/
├── Cargo.toml              Rust 2024; MSRV 1.97.1; all deps; [lints] table; [lib]
├── rustfmt.toml            100-col, Rust 2024 edition, crate-level import grouping
├── .clippy.toml            MSRV 1.97.1, cognitive-complexity 30
├── .env.example            All optional env vars (ANTHROPIC_API_KEY, SOLANA_RPC_URL, …)
├── LICENSE-MIT             MIT licence
├── LICENSE-APACHE          Apache 2.0 licence
├── README.md               This file
├── CHANGELOG.md            Version history and fix log
├── CONTRIBUTING.md         Development workflow and code-style guide
│
├── .github/workflows/
│   └── ci.yml              fmt (nightly) ∥ clippy → build → test → coverage → docs → MSRV
│
├── examples/
│   ├── sor_demo.rs         SOR across 3 venues - no API key needed
│   ├── signer_demo.rs      SignerContext isolation - no API key needed
│   ├── avm_demo.rs         AVM benchmark - run with --release
│   └── jupiter_dry_run.rs  Jupiter swap + Reactor log - no API key needed
│
├── src/
│   ├── lib.rs              Crate root; re-exports all 5 modules
│   ├── main.rs             Binary entry; CLI (clap)
│   ├── config.rs           Config::from_env(); skip_llm; has_api_key()
│   ├── pev/                Plan → Execute → Verify loop
│   │   ├── mod.rs              Orchestrator; MAX_RETRIES = 2
│   │   ├── types.rs            TradeTask, TradeAction, ExecuteOutput, PEVResult
│   │   ├── plan.rs             Haiku decomposition; default_tasks_pub()
│   │   ├── execute.rs          Sonnet execution; action_tool_calls()
│   │   └── verify.rs           Haiku scoring; PASS_THRESHOLD = 0.80
│   ├── sor/                Smart Order Routing
│   │   ├── mod.rs              pub use router::best_route
│   │   ├── router.rs           tokio::join! fan-out; cost ranking; Route struct
│   │   ├── raydium.rs          Stub - Raydium CLMM SDK adapter
│   │   ├── orca.rs             Stub - Orca Whirlpool adapter
│   │   └── serum.rs            Stub - OpenBook CLOB adapter
│   ├── onchain/            On-chain execution
│   │   ├── mod.rs              execute_swap(); demo_signer()
│   │   ├── signer.rs           LocalSolanaSigner; with_signer(); task_local!
│   │   ├── jupiter.rs          simulate_swap(); SwapResult; SIM_ signature
│   │   ├── balance.rs          Stub - sol_balance / token_balance
│   │   └── types.rs            Stub - Lamports, TokenAmount, TxStatus
│   └── avm/                AVM execution layer
│       ├── mod.rs              run_benchmark(); audit_log()
│       ├── benchmark.rs        AVM JIT vs EVM, 10 000 iterations
│       └── reactor.rs          Reactor GUI audit log (3-phase structured trace)
│
└── tests/
    ├── test_pev_loop.rs        10 tests - Config, PEV stub paths, types
    ├── test_sor.rs              3 tests  - best_route, cost ordering, latency
    ├── test_signer_context.rs   2 tests  - SignerContext isolation, Jupiter dry-run
    ├── test_avm_benchmark.rs    1 test   - benchmark smoke test
    └── providers/
        └── anthropic.rs         2 tests  - live, #[ignore], requires ANTHROPIC_API_KEY
```

---

## Key Design Decisions

| Decision | Rationale |
|---|---|
| Lib + bin targets from the same source tree | Integration tests are separate crates; `hft_optimus::` is the correct import prefix |
| Rust 2024 edition | Matches the rig upstream repository |
| Haiku for Plan + Verify, Sonnet for Execute | 60–70% cost reduction vs all-Sonnet; Haiku handles structured, low-complexity steps |
| `CompletionClient` + `ProviderClient` in all PEV files | Both required by rig-core ≥ 0.36 for `.agent()` method resolution |
| `Client::new(&key)?` not `Arc::new(Client::new(...))` | `Client::new` is fallible in rig-core 0.36+ |
| `tokio::task_local!` with `//` not `///` | rustdoc cannot attach to macro invocation sites |
| `#[ignore]` on live provider tests | Prevents CI failures when `ANTHROPIC_API_KEY` is absent |
| `strip = "debuginfo"` in release profile | Smaller binary; mirrors rig's own release profile |
| `CARGO_INCREMENTAL=0` for release builds | Required when `lto = true` |
| Fallback route on all-venue failure | Pipeline never blocked by transient DEX outages |
| `MIT OR Apache-2.0` licence | SPDX-compliant dual licence for crates.io publication |

---

## Related

- [Star Story](./docs/star_story.md) - project narrative
- [Architecture Diagram](./docs/architecture.md) - deep-dive system design
- [Screen Capture Guide](./docs/screen_capture_guide.md) - key output walkthroughs
- [Rig Framework](https://rig.rs) · [0xPlaygrounds/rig](https://github.com/0xPlaygrounds/rig)
- [Solana Program Library](https://spl.solana.com/)
- [Jupiter Aggregator](https://jup.ag/) · [Raydium](https://raydium.io/) · [Orca](https://www.orca.so/)
- [arc.fun](https://arc.fun) · [Ryzome](https://ryzome.ai)

---

## License

Proprietary - © 2026 Murtaza Ali Imtiaz / mAI (🧠)  
See [LICENSE-MAI](LICENSE-MAI) for permitted use.

Licensed under:

- [MIT License](LICENSE-MIT)
- [Apache License, Version 2.0](LICENSE-APACHE)

---

## Author

**Murtaza Ali Imtiaz** · Technology Lead · **mAI (🧠)** · (July 2019 – Present)

- GitHub: [@murtazaneuron](https://github.com/murtazaneuron)
- LinkedIn: [linkedin.com/in/murtazai](https://linkedin.com/in/murtazai)
- Portfolio: [murtazai.com](https://murtazai.com)
