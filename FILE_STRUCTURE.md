# Repository File Structure

```
hft-optimus/
│
│  ── Root tooling & meta ────────────────────────────────────────────
├── Cargo.toml             Rust 2024 edition; all dependencies; lint table
├── Cargo.lock             Committed (binary crate); delete + regenerate on dep changes
├── rustfmt.toml           Code-style rules (100 cols, 2024 edition, crate-level imports)
├── .clippy.toml           Clippy config (MSRV 1.97.1, complexity thresholds)
├── .gitignore             Focused Rust-only ignore file
├── .env.example           Template for ANTHROPIC_API_KEY and other env vars
├── LICENSE                mAI (🧠) proprietary licence (MAI)
├── README.md              Project overview and quick-start
├── CHANGELOG.md           Version history (all 16 bug fixes documented)
├── CONTRIBUTING.md        Dev setup, workflow, code-style, CI description
├── BUG-FIXES.md           Detailed root-cause analysis of all 16 resolved bugs
├── FILE_STRUCTURE.md      This file
│
│  ── GitHub Actions CI ──────────────────────────────────────────────
├── .github/
│   └── workflows/
│       └── ci.yml         fmt (nightly) ∥ clippy → build → test → coverage → docs → MSRV (1.97.1)
│
│  ── Zed IDE config ─────────────────────────────────────────────────
├── .zed/
│   ├── settings.json        rust-analyzer config; format-on-save; inlay hints; import grouping
│   ├── tasks.json           build, test, lint, fmt, coverage, run, examples, CI simulation
│   └── debug.json           CodeLLDB configs for all binary modes, integration tests, coverage
│
│  ── Documentation ──────────────────────────────────────────────────
├── docs/
│   ├── architecture.md      System architecture deep-dive
│   ├── screen_capture_guide.md
│   └── star_story.md
│
│  ── Standalone runnable examples ───────────────────────────────────
│  (cargo run --example <name>; no API key needed except pev_demo)
├── examples/
│   ├── sor_demo.rs          Concurrent SOR across 3 venues; prints winning route
│   ├── signer_demo.rs       SignerContext task-local isolation across 3 tasks
│   ├── avm_demo.rs          AVM vs EVM benchmark (run with --release)
│   └── jupiter_dry_run.rs   Jupiter swap simulation + Reactor audit log
│
│  ── Library source ─────────────────────────────────────────────────
├── src/
│   ├── lib.rs               Crate root; re-exports avm, config, onchain, pev, sor
│   ├── main.rs              Binary entry point; CLI arg parsing (clap)
│   ├── config.rs            Config::from_env(); reads ANTHROPIC_API_KEY etc.
│   │
│   ├── pev/                 Plan → Execute → Verify loop
│   │   ├── mod.rs           Orchestrator: pev::run(), MAX_RETRIES
│   │   ├── types.rs         TradeTask, TradeAction, ExecuteOutput, PEVResult
│   │   ├── plan.rs          Haiku decomposition → Vec<TradeTask>
│   │   ├── execute.rs       Sonnet execution → ExecuteOutput
│   │   └── verify.rs        Haiku scoring → (score, feedback, passed)
│   │
│   ├── sor/                 Smart Order Routing
│   │   ├── mod.rs           pub use router::best_route
│   │   ├── router.rs        Concurrent venue queries; cost-adjusted ranking; Route
│   │   ├── raydium.rs       Stub (future Raydium CLMM SDK adapter)
│   │   ├── orca.rs          Stub (future Orca Whirlpool SDK adapter)
│   │   └── serum.rs         Stub (future OpenBook CLOB adapter)
│   │
│   ├── onchain/             On-chain execution layer
│   │   ├── mod.rs           execute_swap(), demo_signer()
│   │   ├── signer.rs        LocalSolanaSigner; with_signer(); task_local! SignerContext
│   │   ├── jupiter.rs       simulate_swap(); SwapResult
│   │   ├── balance.rs       Stub (future sol_balance / token_balance helpers)
│   │   └── types.rs         Stub (future Lamports, TokenAmount, TxStatus newtypes)
│   │
│   └── avm/                 AVM execution layer
│       ├── mod.rs           run_benchmark(), audit_log()
│       ├── benchmark.rs     AVM JIT vs EVM interpretation (10 000 iterations)
│       └── reactor.rs       Reactor GUI audit log (state-before / exec / state-after)
│
│  ── Integration tests (no API key required) ─────────────────────────
├── tests/
│   ├── test_pev_loop.rs       PEV types, default_tasks_pub, PASS_THRESHOLD
│   ├── test_sor.rs            best_route, cost ordering, latency
│   ├── test_signer_context.rs SignerContext isolation, Jupiter dry-run
│   ├── test_avm_benchmark.rs  Benchmark smoke test
│   │
│   └── providers/             Live provider tests - gated behind #[ignore]
│       └── anthropic.rs       Requires ANTHROPIC_API_KEY; run with --ignored
```

## Key design decisions

| Decision | Rationale |
|---|---|
| Lib + bin targets | Integration tests are external crates; lib exposes `hft_optimus::` |
| Rust 2024 edition | Matches the rig upstream repository |
| `CompletionClient`| Both required by rig-core ≥ 0.36 for `.agent()` |
| `Client::new(...)? ` not `Arc::new(Client::new(...))` | `Client::new` is fallible in 0.36+ |
| `tokio::task_local!` with `//` not `///` | rustdoc cannot attach to macro invocation sites |
| `#[ignore]` on live tests | Prevents CI failures when `ANTHROPIC_API_KEY` is absent |
| `strip = "debuginfo"` in release | Reduces binary size; mirrors rig release profile |
