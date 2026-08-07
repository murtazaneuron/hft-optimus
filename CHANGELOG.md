# Changelog

All notable changes to `hft-core` are documented here.

This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- `cargo-llvm-cov` coverage toolchain: LLVM source-based instrumentation via
  `taiki-e/install-action@v2` + `llvm-tools-preview` component; emits `lcov.info`
  and `coverage-html/` on every CI run (stable matrix leg only)
- `[profile.coverage]` in `Cargo.toml`: inherits test profile with `debug = true`,
  `lto = false`, `opt-level = 0` for accurate instrumented builds
- `.zed/tasks.json`: `§ COVERAGE` section with six self-bootstrapping tasks
  (`workspace`, `summary`, `lcov only`, `open HTML report`, `lib only`,
  `clean profraw artefacts`, `install toolchain`)
- `.zed/debug.json`: four coverage entries accessible from the Run & Debug panel
  (`workspace`, `summary`, `open HTML report`, `clean profraw artefacts`)
- `rustfmt.toml`: `unstable_features = true` + inline `# nightly` annotations on
  all nightly-only options; silences stable rustfmt warnings in CI and locally
- CI `fmt` job: dedicated parallel job running `dtolnay/rust-toolchain@nightly`
  with `rustfmt` component; isolates nightly from the matrix job entirely

### Changed

- **CI – `fmt` job split out**: format check moved from the `ci` matrix job into
  its own `fmt` job (nightly rustfmt); eliminates the toolchain-override / restore
  dance that caused clippy failures and step cancellations
- **CI – `llvm-tools-preview`** added to `dtolnay/rust-toolchain` component list
  in the `ci` job (required by `cargo-llvm-cov`)
- **CI – `rustfmt` removed** from `ci` job component list (now handled by `fmt` job)
- **CI – `target/` removed from cache**: `actions/cache` path list trimmed to
  registry only (`~/.cargo/registry/index`, `cache`, `~/.cargo/git/db`); `target/`
  at ~3 GB caused step-timeout cancellations with no net build-time benefit
- **CI – Codecov upload removed**: `codecov/codecov-action` step and
  `CODECOV_TOKEN` secret dependency removed after Codecov GitHub plugin was
  uninstalled; HTML artifact upload via `actions/upload-artifact@v4` is retained
- **CI – `codecov/codecov-action` upgraded** from `v4` → `v5` (while it was present)
- **CI – `cargo test` step** unchanged; coverage runs as a separate subsequent step
  so test failures are reported independently of coverage failures
- `CONTRIBUTING.md`: prerequisites table updated (nightly, `llvm-tools-preview`,
  `cargo-llvm-cov`); fmt commands updated to `cargo +nightly fmt`; coverage workflow
  documented; CI section rewritten to reflect two-job structure
- `FILE_STRUCTURE.md`: `ci.yml` annotation and `.zed/` block updated
- `README.md`: Codecov badge removed; CI pipeline table updated; prerequisites,
  lint/format, and coverage sections updated to match current toolchain

---

## [0.1.0] - 2025-07-01

### Added

- `examples/sor_demo.rs` - standalone Smart Order Routing demo
- `examples/signer_demo.rs` - `SignerContext` task-local isolation demo
- `examples/avm_demo.rs` - AVM vs EVM micro-benchmark demo
- `examples/jupiter_dry_run.rs` - Jupiter dry-run swap + Reactor audit log demo
- `tests/providers/anthropic.rs` - live Anthropic integration tests (gated behind
  `#[ignore]`)
- `rustfmt.toml` - code-style configuration (mirrors rig upstream)
- `.clippy.toml` - Clippy configuration with MSRV and complexity thresholds
- `CHANGELOG.md` - this file
- `CONTRIBUTING.md` - contribution guide
- `LICENSE-MIT`, `LICENSE-APACHE` - dual MIT / Apache-2.0 licence for crates.io
- `[lib]` section in `Cargo.toml` - explicit library target alongside the binary
- `exclude` list in `Cargo.toml` - keeps IDE config and internal docs out of the
  published crate

### Changed

- `Cargo.toml` - upgraded to Rust **2024 edition**; added `rust-version = "1.97.1"`,
  `[package.metadata.docs.rs]`, and `[lints]` tables; licence changed to
  `MIT OR Apache-2.0` (SPDX-compliant for crates.io); added `authors`, `homepage`,
  `documentation`, and explicit `[lib]` / `exclude` fields
- `src/lib.rs` - expanded crate-level documentation with feature overview and stub
  mode instructions
- `src/pev/{plan,execute,verify}.rs` - expanded `use rig::client::...` import to
  include both `CompletionClient` and `ProviderClient` (required by rig-core ≥ 0.36)
- `src/pev/{plan,execute,verify}.rs` - converted raw-string preamble literals to
  `r"..."` (no `#` delimiter needed; matches 2024 edition style)

### Fixed (historical - see `BUG-FIXES.md` for full root-cause analysis)

| Fix | Summary |
|-----|---------|
| 1 | Removed fictitious `features = ["anthropic", "openai", "cohere"]` from `rig-core` |
| 2 | Upgraded Solana crates to `^3` / `spl-token ^9` to resolve `ed25519-dalek` v1/v2 conflict |
| 3 | Deleted stale `Cargo.lock` with pinned old resolution graph |
| 4 | Replaced `crate::` with `hft_core::` in integration tests |
| 5 | Corrected `format!` positional-arg mismatch in `execute.rs` |
| 6 | Removed unused `rig::tool::Tool` import from `execute.rs` |
| 7 | Merged stray second string literal into the `format!` call in `plan.rs` |
| 8 | Added `pub fn default_tasks_pub` alias callable from integration tests |
| 9 | Populated empty `src/sor/mod.rs` with `pub use router::best_route` |
| 10 | Added `src/lib.rs`; replaced `mod` re-declarations in `main.rs` |
| 11 | Bumped `rig-core` from stale `0.9.1` to `^0.36` |
| 12 | Bumped `reqwest` from `^0.12` to `^0.13` |
| 13 | Renamed `reqwest` feature `rustls-tls` → `rustls` (renamed in 0.13) |
| 14 | Corrected Rust doc comment syntax throughout |
| 15 | Removed `Arc::new(anthropic::Client::new(...))` - `Client::new` is fallible in rig-core 0.36+ |
| 16 | Added `rig::client::CompletionClient` import to all three PEV phase files |
| 17 | Expanded `use rig::{...}` to the multi-line canonical form in all PEV files |
| 18 | Added `rig::client::ProviderClient` to imports in all PEV phase files |
