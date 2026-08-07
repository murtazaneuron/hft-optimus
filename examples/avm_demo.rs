//! AVM vs EVM execution benchmark demo.
//!
//! Runs 10 000 iterations of a JIT-compiled AVM simulation and a heap-allocating
//! EVM bytecode simulation, then prints the speedup ratio.
//!
//! ```text
//! cargo run --example avm_demo --release
//! ```
//!
//! Run with `--release` for meaningful timing numbers; debug builds disable most
//! compiler optimisations and will not reflect real-world speedups.
//!
//! No `ANTHROPIC_API_KEY` is required.

use anyhow::Result;
use tracing_subscriber::EnvFilter;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("hft_core=info"))
        .init();

    hft_core::avm::run_benchmark()?;
    println!("AVM benchmark demo complete.");
    Ok(())
}
