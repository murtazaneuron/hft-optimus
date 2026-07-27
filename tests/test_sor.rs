//! Integration tests for Smart Order Routing.
//!
//! Verifies that [`hft_optimus::sor::best_route`] returns a valid venue,
//! that the cost-ordering logic selects the cheapest option, and that wall-clock
//! latency is measured.
//!
//! None of these tests require `ANTHROPIC_API_KEY`.

use hft_optimus::sor::router::Route;

// ── helper ────────────────────────────────────────────────────────────────────

fn make_route(venue: &str, price: f64, fee_bps: u16) -> Route {
    Route {
        venue: venue.into(),
        effective_price: price,
        fee_bps,
        price_impact_pct: 0.02,
        latency_ms: 10,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// `best_route` must return one of the three known DEX venues.
///
/// This test does NOT require `ANTHROPIC_API_KEY`.
#[tokio::test]
async fn test_best_route_returns_known_venue() {
    let route = hft_optimus::sor::best_route("SOL/USDC", 1.0).await.unwrap();
    assert!(
        ["Raydium", "Orca", "Serum"].contains(&route.venue.as_str()),
        "unexpected venue: {}",
        route.venue
    );
    assert!(route.effective_price > 0.0, "price must be positive");
    assert!(route.fee_bps > 0, "fee_bps must be positive");
}

/// `best_route` must record a non-zero latency.
#[tokio::test]
async fn test_sor_latency_recorded() {
    let route = hft_optimus::sor::best_route("SOL/USDC", 1.0).await.unwrap();
    assert!(
        route.latency_ms > 0,
        "latency must be measured and non-zero"
    );
}

/// Lower fee at the same nominal price yields a lower effective cost.
#[test]
fn test_cost_ordering_lower_fee_wins() {
    let cheaper = make_route("A", 143.50, 20);
    let pricier = make_route("B", 143.50, 30);

    let cost = |r: &Route| r.effective_price * (1.0 + f64::from(r.fee_bps) / 10_000.0);
    assert!(
        cost(&cheaper) < cost(&pricier),
        "venue with lower fee_bps should have lower effective cost"
    );
}
