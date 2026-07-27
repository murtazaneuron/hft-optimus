# Bug Fixes

1. Fix: Removed all the features from rig-core depdendency in Cargo.toml. 
   Root Cause: The features = ["anthropic", "openai", "cohere"] lines in Cargo.toml were fictitious and cargo rejected them.
2. Fix: Upgraded the three Solana crates to 3.x, that was the series where Solana migrated to ed25519-dalek ^2.1.1 (via the new solana-keypair sub-crate). solana-sdk 3.x still re-exported Pubkey, Keypair, and Signer from the same paths, so the signer.rs compiled unchanged. spl-token needed a matching bump to 9.x (the version aligned to solana 3.x).
   Root Cause: solana-sdk 1.18 hard-pinned ed25519-dalek = "=1.0.1" (exact version, v1). The project directly required ed25519-dalek = "^2". Cargo could not unify a v1 exact pin with a v2 requirement, they were different semver epochs and incompatible. Every version in the ^1.18 range had this same pin, so no 1.18.x release could ever resolve.
3. Fix: Deleted Cargo.lock. Cargo regenerated it cleanly on the next cargo build.
   Root Cause: The locked graph pinned the old rig-core 0.9.1 and solana-sdk 1.18 resolutions. 
4. Fix: All crate:: references replaced with hft_optimus::. Integration tests are external crates; crate:: in them refered to the test crate itself, not the project being tested.
   Files: tests/test_pev_loop.rs, test_sor.rs, & test_signer_context.rs
5. Fix: format! had the string "Acceptance criteria: {:?}\nExecute this task now." as the first positional arg (filling in Task ID: {}), then 5 more args for 4 placeholders: compile error argument never used. Reordered to build the prompt correctly.
   File: src/pev/execute.rs
6. Fix: Removed unused rig::tool::Tool import.
   File: src/pev/execute.rs
7. Fix: format!("…{pair}…{amount}…", "Return JSON array only."), the second string literal was being passed as a positional argument with no {} to land in: compiled error argument never used. Merged into one string.
   File: src/pev/plan.rs
8. Fix: default_tasks was private; added pub fn default_tasks alias that the integration test calls.
   File: src/pev/plan.rs
7. Fix: Added pub mod {orca,raydium,router,serum} and pub use router::best_route. main.rs calls sor::best_route(...) 
   File: src/sor/mod.rs
8. Fix: Replaced mod avm; mod config; … with use hft_optimus::{avm, config, onchain, pev, sor};. The binary now re-used the lib's compiled modules instead of redeclaring them.
   File: src/main.rs
9. Fix: Added a library root that re-exports all five modules (avm, config, onchain, pev, sor). Integration tests in tests/ compile as a separate crate, they count not use crate:: to reach into a [[bin]]. Adding a lib target made hft_optimus:: the correct prefix.
   File: New file src/lib.rs
10. Fix: Used rig-core ^0.36. the providers::anthropic API used in the code matched the 0.36 surface
    Root Cause: rig-core 0.9.1 was stale; 
11. Fix: Used reqwest ^0.13
    Root Cause: rig-core 0.36 transitively required reqwest ^0.13; mismatching majors caused two copies and potential API collisions
12. Fix: Changed the reqwest feature to rustls. rustls is also pulled in automatically via the default feature set (default → default-tls → rustls), so never needed to be explicit, we can just write features = ["json"] and TLS comes for free. Keept it explicit as "rustls" made the intent clear.
    Root Cause: `reqwest` with feature `rustls-tls` but `reqwest` does not have that feature.  In reqwest 0.13 the feature was renamed: rustls-tls → rustls.
13. Rust Doc comments added.
14. Fix: Rust Doc Comments Syntax and Semantics corrected.
15. Fix: The Arc was also unnecessary, client consumed immediately by .agent().preamble().build() in the very next line and never shared across tasks, so removed it has no effect on behaviour. The use std::sync::Arc; import was also removed from all three files to keep them warning-free.
    Root Cause: In rig-core 0.36, anthropic::Client::new(&key) was made fallible, it now returns Result<Client<AnthropicExt>, Error> rather than a bare Client. The code was wrapping the call in Arc::new(...), which produced Arc<Result<…>>. Rust's method resolution couldn't find .agent() on that type, hence the E0599.
16. Fix: Added `rig::client::CompletionClient` trait imports to all three PEV phase files.
    Root Cause: In rig-core 0.36+, `.agent()` is a method on the `CompletionClient` trait, not an inherent method on `anthropic::Client`. Without bringing `CompletionClient` into scope, the compiler cannot resolve the method call even though `Client<AnthropicExt>` implements the trait. The official rig documentation and GitHub examples show the canonical import as `use rig::client::CompletionClient;`. Both traits must be in scope.
    Files: src/pev/execute.rs, src/pev/plan.rs, src/pev/verify.rs
17. Fix: Added `rig::client::CompletionClient` to the actual `use` import statement in all three PEV phase files.
    Root Cause: Fix 16 correctly documented the requirement for both `CompletionClient` in the module-level `//!` doc comments, and the module docs were updated to describe both traits. However, the actual `use rig::{...}` import line was only updated to include `CompletionClient` was documented but never imported. At compile time the compiler still cannot resolve `.agent()` because the trait must be *in scope* via a `use` item, not merely mentioned in documentation. The fix expands the import to the multi-line canonical form used in all official rig 0.36 examples:
    ```rust
    use rig::{
        client::CompletionClient,
        completion::Prompt,
        providers::anthropic,
    };
    ```
    Files: src/pev/plan.rs, src/pev/execute.rs, src/pev/verify.rs
18. Fix: Added `rig::client::ProviderClient` to both the module-level `//!` doc bullet list
    and the actual `use` import statement in all three PEV phase files.
    Root Cause: Fix 17 expanded the `use rig::{...}` import to the multi-line form and
    confirmed `CompletionClient` was present, but `ProviderClient` was absent from both the
    module-level documentation bullet list and the `use` statement. The module docs stated
    "both traits" in prose but listed only one trait in the bullet items, giving a misleading
    impression that `CompletionClient` alone was sufficient. At runtime in rig-core 0.36+ the
    provider-client construction pattern requires `ProviderClient` to also be in scope via a
    `use` item - documentation mentions have no effect on method resolution. The canonical
    multi-line import matching all official rig 0.36 examples is:
    ```rust
    use rig::{
        client::{CompletionClient, ProviderClient},
        completion::Prompt,
        providers::anthropic,
    };
    ```
    Files: src/pev/plan.rs, src/pev/execute.rs, src/pev/verify.rs
