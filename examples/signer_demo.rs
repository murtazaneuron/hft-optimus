//! `SignerContext` isolation demo.
//!
//! Spawns three concurrent Tokio tasks and demonstrates that each task holds an
//! independent `LocalSolanaSigner` in its task-local storage.  No keypair leaks
//! between tasks even though they overlap in wall-clock time.
//!
//! ```text
//! cargo run --example signer_demo
//! ```
//!
//! No `ANTHROPIC_API_KEY` is required; no on-chain call is made.

use anyhow::Result;
use hft_core::config::Config;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("hft_core=debug"))
        .init();

    // Config::from_env() no longer requires ANTHROPIC_API_KEY.
    // When the key is absent, skip_llm is set to true automatically and all
    // LLM-free paths (including the signer demo) work without any special setup.
    let cfg = Config::from_env()?;

    hft_core::onchain::demo_signer(&cfg).await?;
    println!("SignerContext isolation demo complete.");
    Ok(())
}
