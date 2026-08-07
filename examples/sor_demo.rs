//! Smart Order Routing demo.
//!
//! Runs a single concurrent venue comparison across Raydium, Orca, and Serum
//! and prints the winning route with its cost-adjusted price.
//!
//! ```text
//! cargo run --example sor_demo
//! ```

use anyhow::Result;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new("hft_core=info"))
        .init();

    let route = hft_core::sor::best_route("SOL/USDC", 1.0).await?;

    println!("┌─────────────────────────────────────┐");
    println!("│  SOR Result                         │");
    println!("├─────────────────────────────────────┤");
    println!("│  Venue:         {:>20} │", route.venue);
    println!("│  Price:         {:>17.4} USDC │", route.effective_price);
    println!("│  Fee:           {:>18} bps │", route.fee_bps);
    println!(
        "│  Eff. cost:     {:>17.4} USDC │",
        route.effective_price * (1.0 + f64::from(route.fee_bps) / 10_000.0)
    );
    println!("│  Latency:       {:>17} ms  │", route.latency_ms);
    println!("└─────────────────────────────────────┘");

    Ok(())
}
