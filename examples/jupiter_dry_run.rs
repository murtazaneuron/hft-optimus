//! Jupiter swap dry-run demo.
//!
//! Simulates a SOL → USDC swap through the Orca venue without making any real
//! on-chain call. Demonstrates the `SwapResult` structure and the Reactor GUI
//! audit log output.
//!
//! ```text
//! cargo run --example jupiter_dry_run
//! ```
//!
//! No `ANTHROPIC_API_KEY` is required; no on-chain call is made.

use anyhow::Result;
use hft_optimus::sor::router::Route;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("hft_optimus=info"))
        .init();

    let route = Route {
        venue: "Orca".into(),
        effective_price: 143.48,
        fee_bps: 30,
        price_impact_pct: 0.02,
        latency_ms: 9,
    };

    let swap = hft_optimus::onchain::jupiter::simulate_swap(&route, 1.0, true).await?;

    println!("┌──────────────────────────────────────────┐");
    println!("│  Jupiter Dry-Run Result                  │");
    println!("├──────────────────────────────────────────┤");
    println!("│  Signature:  {}  │", swap.simulated_sig);
    println!(
        "│  Input:      {:.6} SOL                  │",
        swap.input_amount
    );
    println!(
        "│  Output:     {:.4} USDC                 │",
        swap.output_amount
    );
    println!("│  Fee:        {:.6} SOL                  │", swap.fee_paid);
    println!("│  Dry run:    {:?}                     │", swap.is_dry_run);
    println!("└──────────────────────────────────────────┘");

    println!("\n── Reactor GUI Audit Log ───────────────────");
    hft_optimus::avm::audit_log(&route, &swap)?;

    Ok(())
}
