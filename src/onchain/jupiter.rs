//! Jupiter swap simulation (dry-run).
//!
//! Provides [`simulate_swap`], which mimics the Jupiter Aggregator V6 API
//! without sending any real transaction to the network. In production, replace
//! this module with `rig_onchain_kit::tools::JupiterSwap` once
//! `rig-onchain-kit` is published to crates.io.
//!
//! ## Dry-run mode
//!
//! When `dry_run = true` (the platform default via [`crate::config::Config`]),
//! [`simulate_swap`]:
//!
//! 1. Computes `output_amount = input_amount × effective_price`.
//! 2. Computes `fee_paid = input_amount × fee_bps / 10_000`.
//! 3. Generates a simulated transaction signature prefixed with `SIM_`.
//!
//! No RPC call is made. The returned [`SwapResult`] is structurally identical
//! to what a real Jupiter swap returns, making it straightforward to upgrade
//! to live mode later.

use anyhow::Result;
use tracing::info;

use crate::sor::router::Route;

/// The result of a Jupiter swap - real or simulated.
#[derive(Debug, Clone)]
pub struct SwapResult {
    /// Transaction signature on the Solana network.
    ///
    /// Prefixed with `SIM_` followed by a random 16-character hex string when
    /// produced in dry-run mode.
    pub simulated_sig: String,

    /// Amount of the base token sent into the swap (in token units).
    pub input_amount: f64,

    /// Amount of the quote token received after fees (in token units).
    pub output_amount: f64,

    /// Absolute fee paid, denominated in the input token.
    pub fee_paid: f64,

    /// `true` when this result was produced by a simulation; `false` when a
    /// real transaction was broadcast.
    pub is_dry_run: bool,
}

/// Simulate (or execute live) a Jupiter swap for the given route and amount.
///
/// In dry-run mode the function is purely computational - no network call is
/// made and no transaction is signed. In live mode the function returns `Err`
/// because real transaction signing is not implemented in this demo build.
///
/// # Arguments
///
/// * `route`   - The SOR-selected venue (venue name, effective price, fee).
/// * `amount`  - Base-token amount to swap.
/// * `dry_run` - `true` to simulate only; `false` to attempt a live transaction (currently
///   unimplemented in this demo).
///
/// # Errors
///
/// Returns `Err` when `dry_run` is `false`, because live-mode signing is not
/// yet wired up.
#[allow(clippy::unused_async)] // TODO: will await Jupiter RPC call once dry_run path is removed
pub async fn simulate_swap(route: &Route, amount: f64, dry_run: bool) -> Result<SwapResult> {
    info!(
        venue  = %route.venue,
        price  = route.effective_price,
        amount,
        dry_run,
        "[JUPITER] Simulating swap"
    );

    if !dry_run {
        // Production path: delegate to rig-onchain-kit JupiterSwap tool.
        // let agent = create_solana_agent();
        // let result = agent.prompt("Swap 1 SOL to USDC via Jupiter").await?;
        anyhow::bail!("Live mode not enabled in demo. Omit --live or set DRY_RUN=true.");
    }

    let output = amount * route.effective_price;
    let fee = amount * (f64::from(route.fee_bps) / 10_000.0);
    let sig = format!("SIM_{:016x}", rand::random::<u64>());

    let result = SwapResult {
        simulated_sig: sig,
        input_amount: amount,
        output_amount: output,
        fee_paid: fee,
        is_dry_run: true,
    };

    info!(
        sig    = %result.simulated_sig,
        input  = result.input_amount,
        output = result.output_amount,
        fee    = result.fee_paid,
        "[JUPITER] Swap simulation complete (DRY RUN)"
    );

    Ok(result)
}
