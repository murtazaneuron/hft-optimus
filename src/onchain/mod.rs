//! On-chain execution layer.
//!
//! Exposes two public entry points consumed by `crate::main`:
//!
//! * [`execute_swap`] - wraps a Jupiter swap simulation inside an isolated
//!   [`signer::LocalSolanaSigner`] context and returns a [`jupiter::SwapResult`].
//! * [`demo_signer`] - spawns three concurrent tasks to demonstrate that the task-local signer
//!   storage is fully isolated per task.
//!
//! ## Security boundary
//!
//! All on-chain operations are wrapped in [`signer::with_signer`], which uses
//! `tokio::task_local!` to scope the active keypair to exactly one async task.
//! This mirrors the `rig-onchain-kit` `SignerContext` pattern and prevents
//! concurrent tasks from accidentally sharing or overwriting each other's
//! signing credentials.

pub mod balance;
pub mod jupiter;
pub mod signer;
pub mod types;

use anyhow::Result;
use jupiter::SwapResult;
use tracing::info;

use crate::{config::Config, sor::router::Route};

/// Execute a swap for the given route inside an isolated signer context.
///
/// Loads a [`signer::LocalSolanaSigner`] from the environment, logs the
/// public key, then runs the Jupiter simulation inside [`signer::with_signer`]
/// to guarantee the keypair is scoped to this task only.
///
/// The `live` flag is inverted before being passed to
/// [`jupiter::simulate_swap`] because that function's parameter is named
/// `dry_run` - the logical complement of `live`.
///
/// # Arguments
///
/// * `cfg`   - Runtime configuration (provides the RPC URL and dry-run flag).
/// * `route` - The SOR-selected venue and price quote to execute against.
/// * `live`  - `true` to attempt a real transaction; `false` (default) for dry-run simulation.
///
/// # Errors
///
/// Returns `Err` if the signer context setup or the Jupiter simulation fails.
pub async fn execute_swap(cfg: &Config, route: &Route, live: bool) -> Result<SwapResult> {
    let _ = cfg; // reserved for future RPC client construction
    let signer = signer::LocalSolanaSigner::from_env();
    let pubkey = signer.pubkey();
    info!(%pubkey, "[ONCHAIN] SignerContext: signer loaded");

    // Wrap all on-chain operations in SignerContext for task-local isolation.
    signer::with_signer(signer, || async {
        let result = jupiter::simulate_swap(route, 1.0, !live).await?;
        Ok(result)
    })
    .await
}

/// Demonstrate [`signer::with_signer`] isolation across three concurrent tasks.
///
/// Delegates directly to [`signer::demo_signer`].
///
/// # Arguments
///
/// * `cfg` - Runtime configuration (reserved for future credential loading).
///
/// # Errors
///
/// Returns `Err` if any spawned task panics or its [`tokio::task::JoinHandle`]
/// fails.
pub async fn demo_signer(cfg: &Config) -> Result<()> {
    signer::demo_signer(cfg).await
}
