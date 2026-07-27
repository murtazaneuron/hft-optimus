//! Smart Order Routing (SOR).
//!
//! Queries Raydium, Orca, and Serum concurrently, then selects the execution
//! venue with the lowest **cost-adjusted price**:
//!
//! ```text
//! effective_cost = price × (1 + fee_bps / 10_000)
//! ```
//!
//! The winning [`router::Route`] is returned to the caller and forwarded to
//! the on-chain execution layer.
//!
//! ## Extending to production
//!
//! The venue adapter files (`raydium.rs`, `orca.rs`, `serum.rs`) are stubs.
//! Replace the mock `tokio::sleep` + hard-coded prices in [`router`] with real
//! SDK calls, e.g. the Raydium CLMM SDK or Orca's Whirlpool API, and move
//! each `query_*` function into its respective adapter file.

pub mod orca;
pub mod raydium;
pub mod router;
pub mod serum;

/// Re-export [`router::best_route`] at the module root for ergonomic use:
///
/// ```rust,ignore
/// let route = sor::best_route("SOL/USDC", 1.0).await?;
/// ```
pub use router::best_route;
