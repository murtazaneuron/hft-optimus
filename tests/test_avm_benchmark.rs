//! Integration test for the AVM execution benchmark.
//!
//! Verifies that [`hft_core::avm::run_benchmark`] completes without
//! panicking or returning an error. The test does not assert on the numeric
//! speedup ratio because that depends on hardware and scheduler state.

/// `run_benchmark` must complete successfully on all platforms.
#[test]
fn test_benchmark_completes_without_error() {
    let result = hft_core::avm::run_benchmark();
    assert!(
        result.is_ok(),
        "AVM benchmark returned an unexpected error: {result:?}"
    );
}
