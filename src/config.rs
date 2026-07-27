//! Runtime configuration loaded from environment variables.
//!
//! All fields are read at startup via [`Config::from_env`].
//! `ANTHROPIC_API_KEY` is **optional**: if absent, [`Config::skip_llm`] is set
//! to `true` and every PEV phase runs in stub/offline mode.  All other
//! subsystems (SOR, `SignerContext`, AVM benchmark) are unaffected and always
//! work.
//!
//! `skip_llm` can also be forced on explicitly via the `SKIP_LLM=1` env var
//! or the `--skip-llm` CLI flag (see `main.rs`).
//!
//! ## Environment variables
//!
//! | Variable | Required | Default |
//! |---|---|---|
//! | `ANTHROPIC_API_KEY` | ❌ | `""` (activates stub mode) |
//! | `SKIP_LLM` | ❌ | `false` |
//! | `SOLANA_RPC_URL` | ❌ | `https://api.devnet.solana.com` |
//! | `SOLANA_PRIVATE_KEY` | ❌ | `DEMO_KEY_PLACEHOLDER` |
//! | `DRY_RUN` | ❌ | `true` |

use anyhow::Result;

/// Global runtime configuration for the HFT platform.
///
/// Constructed once at startup via [`Config::from_env`] and shared by
/// reference across all subsystems.  Cloning is cheap - all fields are either
/// `String` or `bool`.
#[derive(Debug, Clone)]
pub struct Config {
    /// Anthropic API key forwarded to every `rig-core` client.
    ///
    /// Empty when `ANTHROPIC_API_KEY` is not set.  Check [`Config::has_api_key`]
    /// before constructing an LLM client; use [`Config::skip_llm`] as the
    /// single gate for all PEV stub paths.
    pub anthropic_api_key: String,

    /// When `true` every PEV phase (Plan / Execute / Verify) uses its
    /// offline stub instead of calling the Anthropic API.
    ///
    /// Set automatically when `ANTHROPIC_API_KEY` is absent or when
    /// `SKIP_LLM=1` is set.  Also set programmatically by `main.rs` when
    /// the `--skip-llm` CLI flag is passed.
    pub skip_llm: bool,

    /// Solana JSON-RPC endpoint.  Defaults to Solana devnet.
    pub solana_rpc_url: String,

    /// Base-58 encoded Solana keypair used for signing transactions.
    ///
    /// In production this should be loaded from a secrets manager.  The demo
    /// falls back to a freshly generated random keypair when this variable is
    /// absent.
    pub solana_private_key: String,

    /// When `true` (the default), all on-chain operations are simulated and no
    /// real transactions are signed or broadcast.
    ///
    /// Pass `--live` on the CLI to set this to `false`.
    pub dry_run: bool,
}

impl Config {
    /// Construct a [`Config`] from the process environment.
    ///
    /// This function is **infallible**: it never returns `Err` because every
    /// variable has a safe default.  Call [`dotenvy::dotenv`] before this
    /// function to pick up `.env` variables; `main.rs` does this automatically.
    ///
    /// When `ANTHROPIC_API_KEY` is absent [`Config::skip_llm`] is set to
    /// `true` so callers do not need to check separately.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use hft_optimus::config::Config;
    ///
    /// let cfg = Config::from_env().unwrap();
    /// // Works even without ANTHROPIC_API_KEY in the environment.
    /// ```
    pub fn from_env() -> Result<Self> {
        let anthropic_api_key = std::env::var("ANTHROPIC_API_KEY").unwrap_or_default();

        // skip_llm is true when:
        //   • no API key is available, OR
        //   • SKIP_LLM=1 is explicitly set in the environment.
        let env_skip = std::env::var("SKIP_LLM").is_ok_and(|v| v == "1" || v == "true");
        let skip_llm = anthropic_api_key.is_empty() || env_skip;

        Ok(Self {
            anthropic_api_key,
            skip_llm,
            solana_rpc_url: std::env::var("SOLANA_RPC_URL")
                .unwrap_or_else(|_| "https://api.devnet.solana.com".to_string()),
            solana_private_key: std::env::var("SOLANA_PRIVATE_KEY")
                .unwrap_or_else(|_| "DEMO_KEY_PLACEHOLDER".to_string()),
            dry_run: std::env::var("DRY_RUN").map_or(true, |v| v == "true" || v == "1"),
        })
    }

    /// Returns `true` when a non-empty `ANTHROPIC_API_KEY` is available.
    pub fn has_api_key(&self) -> bool {
        !self.anthropic_api_key.is_empty()
    }
}
