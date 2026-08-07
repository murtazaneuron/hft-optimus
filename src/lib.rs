//! # hft-core
//!
//! **LLM-driven high-frequency trading platform on Solana**, built on
//! [Rig (ARC)](https://rig.rs) - the Rust Inference Gateway.
//!
//! Combines agentic LLM pipelines with on-chain `DeFi` execution on Solana,
//! enabling statefully supervised, multi-step agent workflows with full PEV
//! loop governance.
//!
//! > **mAI (🧠)** · Technology Lead: Murtaza Ali Imtiaz (July 2019 – present)
//!
//! ## Architecture
//!
//! ```text
//! ┌──────────────────────────────────────────────────────┐
//! │                  hft-core                  │
//! ├──────────┬──────────┬───────────┬─────────┬──────────┤
//! │   pev    │   sor    │  onchain  │   avm   │  config  │
//! │ Plan –   │ Smart    │ Jupiter   │ AVM     │ Env      │
//! │ Execute–│ Order    │ swap +    │ bench-  │ loader   │
//! │ Verify  │ Routing  │ Signer    │ mark    │          │
//! └──────────┴──────────┴───────────┴─────────┴──────────┘
//! ```
//!
//! ## Full pipeline
//!
//! 1. **PEV loop** ([`pev`]) - Haiku decomposes the trade into four [`pev::types::TradeTask`]
//!    objects; Sonnet executes each one using tool calls; Haiku verifies the output (pass ≥ 0.80).
//!    Up to two retries on failure.
//! 2. **SOR** ([`sor`]) - Raydium, Orca, and Serum are queried concurrently; the lowest
//!    cost-adjusted venue wins.
//! 3. **On-chain** ([`onchain`]) - Jupiter swap is simulated (dry-run by default) inside an
//!    isolated [`onchain::signer::LocalSolanaSigner`] context.
//! 4. **AVM audit** ([`avm`]) - Reactor GUI audit log records state-before, execution details, and
//!    state-after for every swap.
//!
//! ## Quick start
//!
//! ```text
//! cp .env.example .env   # set ANTHROPIC_API_KEY (optional - stub mode works without it)
//! cargo run --release -- --mode full --pair SOL/USDC --amount 1.0
//! ```
//!
//! ## Offline / stub mode
//!
//! All subsystems work without an `ANTHROPIC_API_KEY`.  When a key is absent
//! (or `--skip-llm` is passed), every PEV phase substitutes a deterministic
//! offline stub:
//!
//! ```text
//! SKIP_LLM=1 cargo run --release -- --mode full
//! ```
//!
//! ## Feature flags
//!
//! This crate has no optional feature flags.  All subsystems are always compiled.
//!
//! ## Crate status
//!
//! The venue adapters (`raydium.rs`, `orca.rs`, `serum.rs`) and the balance
//! query helpers (`balance.rs`, `types.rs` in `onchain`) are documented stubs
//! awaiting real SDK integration.  The PEV loop, SOR router, `SignerContext`,
//! Jupiter dry-run simulation, and AVM benchmark are fully implemented.

pub mod avm;
pub mod config;
pub mod onchain;
pub mod pev;
pub mod sor;
