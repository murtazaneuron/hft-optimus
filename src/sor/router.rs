//! Core SOR logic - concurrent venue queries and cost-adjusted ranking.
//!
//! [`best_route`] fans out to three venue adapters in parallel using
//! [`tokio::join!`], collects the successful results, sorts them by
//! effective cost, and returns the cheapest [`Route`]. End-to-end wall-clock
//! latency is measured and stored in [`Route::latency_ms`].

use std::time::Instant;

use anyhow::Result;
use tracing::info;

/// A price quote from a single DEX venue - the core output of a venue query.
///
/// Returned by each venue adapter and compared by [`best_route`] to select
/// the optimal execution path.
#[derive(Debug, Clone)]
pub struct Route {
    /// Human-readable venue name, e.g. `"Raydium"`, `"Orca"`, or `"Serum"`.
    pub venue: String,

    /// Mid-market or quoted price of the base token in the quote currency.
    pub effective_price: f64,

    /// Venue trading fee in basis points (1 bps = 0.01 %).
    pub fee_bps: u16,

    /// Estimated price impact of the requested trade size, as a percentage.
    pub price_impact_pct: f64,

    /// Wall-clock latency from the start of the SOR call to venue selection,
    /// in milliseconds. Set to `0` by each adapter and overwritten by
    /// [`best_route`] once all queries complete.
    pub latency_ms: u128,
}

/// Query all three venues concurrently and return the lowest-cost [`Route`].
///
/// Venues are queried in parallel via [`tokio::join!`]. Failed queries are
/// silently skipped (their `Err` values are discarded). If every query fails,
/// `fallback_route` is returned so the pipeline is never blocked.
///
/// **Cost metric:** `price × (1 + fee_bps / 10_000)`. This accounts for both
/// the quoted price and the trading fee, giving a true all-in execution cost.
///
/// # Arguments
///
/// * `pair`   - Trading pair, e.g. `"SOL/USDC"`.
/// * `amount` - Base-token amount (passed to venue adapters for slippage estimation in production;
///   informational in this demo).
///
/// # Errors
///
/// Currently infallible - failures from individual venue adapters are absorbed
/// and the fallback route is used if all adapters fail.
pub async fn best_route(pair: &str, amount: f64) -> Result<Route> {
    info!(pair, amount, "[SOR] Starting route comparison");
    let t0 = Instant::now();

    let (raydium, orca, serum) = tokio::join!(
        query_raydium(pair, amount),
        query_orca(pair, amount),
        query_serum(pair, amount),
    );

    let mut candidates = vec![];
    if let Ok(r) = raydium {
        candidates.push(r);
    }
    if let Ok(r) = orca {
        candidates.push(r);
    }
    if let Ok(r) = serum {
        candidates.push(r);
    }

    // Sort ascending by effective cost so `next()` yields the cheapest venue.
    candidates.sort_by(|a, b| {
        let cost_a = a.effective_price * (1.0 + f64::from(a.fee_bps) / 10_000.0);
        let cost_b = b.effective_price * (1.0 + f64::from(b.fee_bps) / 10_000.0);
        cost_a
            .partial_cmp(&cost_b)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    let mut best = candidates
        .into_iter()
        .next()
        .unwrap_or_else(|| fallback_route(pair));
    best.latency_ms = t0.elapsed().as_millis();

    info!(
        venue      = %best.venue,
        price      = best.effective_price,
        fee_bps    = best.fee_bps,
        latency_ms = best.latency_ms,
        "[SOR] Best route selected"
    );
    Ok(best)
}

/// Query the Raydium CLMM pool for a price quote.
///
/// **Demo stub.** Returns a hard-coded price after a simulated 12 ms network
/// round-trip. Replace with the Raydium SDK in production.
async fn query_raydium(_pair: &str, _amount: f64) -> Result<Route> {
    tokio::time::sleep(std::time::Duration::from_millis(12)).await;
    Ok(Route {
        venue: "Raydium".into(),
        effective_price: 143.52,
        fee_bps: 25,
        price_impact_pct: 0.03,
        latency_ms: 0,
    })
}

/// Query the Orca Whirlpool for a price quote.
///
/// **Demo stub.** Returns a hard-coded price after a simulated 9 ms network
/// round-trip. Replace with the Orca SDK in production.
async fn query_orca(_pair: &str, _amount: f64) -> Result<Route> {
    tokio::time::sleep(std::time::Duration::from_millis(9)).await;
    Ok(Route {
        venue: "Orca".into(),
        effective_price: 143.48,
        fee_bps: 30,
        price_impact_pct: 0.02,
        latency_ms: 0,
    })
}

/// Query the Serum / `OpenBook` central limit order book for a price quote.
///
/// **Demo stub.** Returns a hard-coded price after a simulated 15 ms network
/// round-trip. Replace with the `OpenBook` SDK in production.
async fn query_serum(_pair: &str, _amount: f64) -> Result<Route> {
    tokio::time::sleep(std::time::Duration::from_millis(15)).await;
    Ok(Route {
        venue: "Serum".into(),
        effective_price: 143.61,
        fee_bps: 20,
        price_impact_pct: 0.05,
        latency_ms: 0,
    })
}

/// Emergency fallback route used when all venue queries fail.
///
/// Returns a conservative Raydium quote so the pipeline can continue in
/// degraded mode rather than returning an error.
fn fallback_route(_pair: &str) -> Route {
    Route {
        venue: "Raydium-fallback".into(),
        effective_price: 143.50,
        fee_bps: 25,
        price_impact_pct: 0.03,
        latency_ms: 0,
    }
}
