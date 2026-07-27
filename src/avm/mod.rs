//! AVM (Agave Virtual Machine) execution layer.
//!
//! mAI (🧠) - Technology Lead: Murtaza Ali Imtiaz
//!
//! Contains two sub-modules that together demonstrate the performance and
//! auditability story of the Agave runtime inside the HFT platform:
//!
//! * `benchmark` - micro-benchmark comparing AVM JIT-compiled execution against EVM bytecode-style
//!   interpretation, demonstrating the ~8–12× throughput advantage of the Agave runtime.
//! * `reactor` - Reactor GUI audit-log simulation, emitting a structured before/after log of a
//!   smart contract deployment so operators get a human-readable, per-trade execution trace.
//!
//! ## Public API
//!
//! | Function | Description |
//! |---|---|
//! | [`run_benchmark`] | Run 10 000-iteration AVM vs EVM benchmark |
//! | [`audit_log`] | Emit Reactor GUI audit log for a completed swap |

pub mod benchmark;
pub mod reactor;

use anyhow::Result;

use crate::{onchain::jupiter::SwapResult, sor::router::Route};

/// Run the AVM vs EVM execution benchmark and log the speedup factor.
///
/// Delegates to [`benchmark::run`].
///
/// # Errors
///
/// Currently infallible; returns `Ok(())`. The signature uses [`Result`] for
/// forward-compatibility with real AVM instrumentation hooks.
pub fn run_benchmark() -> Result<()> {
    benchmark::run()
}

/// Emit a Reactor GUI audit log entry for the given route and swap result.
///
/// Logs state-before, execution details, and state-after at `INFO` level so
/// the output is visible in the default tracing subscriber configuration.
///
/// Delegates to [`reactor::emit_audit_log`].
///
/// # Arguments
///
/// * `route` - The SOR-selected execution venue and price quote.
/// * `swap`  - The completed swap result from the Jupiter simulation.
///
/// # Errors
///
/// Currently infallible.
pub fn audit_log(route: &Route, swap: &SwapResult) -> Result<()> {
    reactor::emit_audit_log(route, swap)
}
