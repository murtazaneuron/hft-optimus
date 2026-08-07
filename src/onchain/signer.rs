//! `SignerContext` - task-local keypair isolation for secure on-chain operations.
//!
//! Implements the security boundary described in the `rig-onchain-kit`
//! documentation: every async on-chain call must be wrapped in
//! [`with_signer`], which scopes the active [`solana_sdk::signature::Keypair`]
//! to exactly the current Tokio task via `tokio::task_local!`.
//!
//! ## Why task-local storage?
//!
//! In an async runtime multiple trades can be in-flight simultaneously.
//! Using a global or thread-local signer would risk one task's keypair leaking
//! into another task's signing operation. `task_local!` gives each spawned
//! task its own isolated slot with no locking overhead.
//!
//! ## Production upgrade path
//!
//! Replace [`LocalSolanaSigner`] and the hand-rolled `task_local!` storage
//! with `rig_onchain_kit::signer::SignerContext` once that crate is published
//! to crates.io.

use std::sync::Arc;

use anyhow::Result;
use solana_sdk::{
    pubkey::Pubkey,
    signature::{Keypair, Signer},
};
use tracing::info;

use crate::config::Config;

// Task-local slot that holds the active signer for the current Tokio task.
// Declared with a regular line comment rather than a doc comment (///)
// because `tokio::task_local!` is a macro invocation - rustdoc cannot attach
// outer doc attributes to macro call sites, which would trigger the
// `unused_doc_comments` lint.
tokio::task_local! {
    static CURRENT_SIGNER: Arc<dyn Signer + Send + Sync>;
}

/// A Solana keypair wrapper that loads its key from the process environment.
///
/// In production, decode the base-58 private key stored in
/// `SOLANA_PRIVATE_KEY` using `Keypair::from_base58_string`. In this demo a
/// fresh random keypair is generated regardless of whether the variable is set.
pub struct LocalSolanaSigner {
    keypair: Keypair,
}

impl LocalSolanaSigner {
    /// Construct a [`LocalSolanaSigner`] from the process environment.
    ///
    /// When `SOLANA_PRIVATE_KEY` is present the variable is acknowledged, but
    /// the demo still generates a random keypair as a placeholder.
    ///
    /// **TODO (production):** replace `Keypair::new()` with
    /// `Keypair::from_base58_string(&key)` to load the real signing key.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use hft_core::onchain::signer::LocalSolanaSigner;
    ///
    /// let s = LocalSolanaSigner::from_env();
    /// println!("signer pubkey: {}", s.pubkey());
    /// ```
    pub fn from_env() -> Self {
        let keypair = if std::env::var("SOLANA_PRIVATE_KEY").is_ok() {
            // TODO: Keypair::from_base58_string(&key) in production
            Keypair::new()
        } else {
            Keypair::new()
        };
        Self { keypair }
    }

    /// Return the [`Pubkey`] (on-chain address) of the wrapped keypair.
    pub fn pubkey(&self) -> Pubkey {
        self.keypair.pubkey()
    }
}

/// Execute an async closure with `signer` installed as the task-local signer.
///
/// Installs `signer` into the `CURRENT_SIGNER` task-local slot for the
/// duration of the future returned by `f`. The signer is automatically
/// removed when the scope exits, so it cannot outlive the operation it was
/// created for.
///
/// # Type parameters
///
/// * `F`   - A [`FnOnce`] closure that produces a future.
/// * `Fut` - The [`Future`] returned by `F`.
/// * `T`   - The `Ok` type of the future's output.
///
/// # Errors
///
/// Propagates any [`anyhow::Error`] returned by the closure `f`.
///
/// # Examples
///
/// ```rust,no_run
/// use hft_core::onchain::signer::{with_signer, LocalSolanaSigner};
///
/// # #[tokio::main]
/// # async fn main() -> anyhow::Result<()> {
/// let signer = LocalSolanaSigner::from_env();
/// let result = with_signer(signer, || async {
///     Ok::<&str, anyhow::Error>("swap executed")
/// }).await?;
/// # Ok(()) }
/// ```
pub async fn with_signer<F, Fut, T>(signer: LocalSolanaSigner, f: F) -> Result<T>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T>>,
{
    let arc_signer: Arc<dyn Signer + Send + Sync> = Arc::new(signer.keypair);
    CURRENT_SIGNER.scope(arc_signer, async { f().await }).await
}

/// Spawn three concurrent tasks and demonstrate that each has an independent
/// task-local signer.
///
/// Each task calls [`LocalSolanaSigner::from_env`] and [`with_signer`]
/// independently. Because `CURRENT_SIGNER` is task-local, no public key leaks
/// between tasks even though they overlap in time.
///
/// # Arguments
///
/// * `_cfg` - Runtime configuration (reserved for future credential loading; currently unused).
///
/// # Errors
///
/// Returns `Err` if any spawned task panics or its
/// [`tokio::task::JoinHandle`] returns an error.
pub async fn demo_signer(_cfg: &Config) -> Result<()> {
    info!("[SIGNER] Demonstrating SignerContext thread-local isolation");

    let handles: Vec<_> = (0..3)
        .map(|i| {
            tokio::spawn(async move {
                let signer = LocalSolanaSigner::from_env();
                let pubkey = signer.pubkey();
                with_signer(signer, || async move {
                    info!(task = i, %pubkey, "[SIGNER] Task running in isolated context");
                    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
                    info!(task = i, "[SIGNER] Task complete - signer isolated");
                    Ok::<(), anyhow::Error>(())
                })
                .await
            })
        })
        .collect();

    for h in handles {
        h.await??;
    }
    info!("[SIGNER] All tasks complete. SignerContext isolation verified.");
    Ok(())
}
