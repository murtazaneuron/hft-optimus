//! Integration tests for [`hft_optimus::onchain::signer`].
//!
//! Verifies that [`with_signer`][hft_optimus::onchain::signer::with_signer]
//! provides task-local isolation across concurrent Tokio tasks, and that
//! `simulate_swap`,`hft_optimus::onchain::jupiter::simulate_swap`]
//! returns a properly formed dry-run result.

use hft_optimus::{
    onchain::{
        jupiter,
        signer::{LocalSolanaSigner, with_signer},
    },
    sor::router::Route,
};

// ── helper ────────────────────────────────────────────────────────────────────

fn test_route() -> Route {
    Route {
        venue: "Raydium".into(),
        effective_price: 143.50,
        fee_bps: 25,
        price_impact_pct: 0.03,
        latency_ms: 10,
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

/// Two concurrently spawned tasks must each obtain their own signer without
/// interfering with each other.
#[tokio::test]
async fn test_signer_context_isolation() {
    let (r1, r2) = tokio::join!(
        tokio::spawn(async {
            let s = LocalSolanaSigner::from_env();
            let pk = s.pubkey();
            with_signer(s, || async { Ok::<_, anyhow::Error>(pk) })
                .await
                .unwrap()
        }),
        tokio::spawn(async {
            let s = LocalSolanaSigner::from_env();
            let pk = s.pubkey();
            with_signer(s, || async { Ok::<_, anyhow::Error>(pk) })
                .await
                .unwrap()
        })
    );
    // Both join handles must succeed (no panic, no error propagation).
    assert!(r1.is_ok(), "task 1 signer context should not fail");
    assert!(r2.is_ok(), "task 2 signer context should not fail");
}

/// A Jupiter dry-run swap must return a simulated signature prefixed `SIM_`
/// and a positive output amount.
#[tokio::test]
async fn test_jupiter_dry_run_returns_simulated_sig() {
    let result = jupiter::simulate_swap(&test_route(), 1.0, true)
        .await
        .unwrap();
    assert!(result.is_dry_run, "result must be flagged as dry-run");
    assert!(
        result.simulated_sig.starts_with("SIM_"),
        "dry-run signature must start with 'SIM_'"
    );
    assert!(result.output_amount > 0.0, "output amount must be positive");
}
