# hft-optimus

## Key Outputs (screen capture)

Running `cargo run -- --mode full` produces structured logs showing:

1. **PEV Loop**: Plan decomposed into 4 atomic tasks → Execute with tool calls
   → Verify score ≥ 0.80 → PASS
2. **Smart Order Routing**: Raydium/Orca/Serum compared concurrently → best
   venue selected with latency in milliseconds logged
3. **SignerContext**: 3 concurrent tasks each isolated in their own signer context
4. **Jupiter swap**: dry-run simulation with SIM_xxxxxxxxxxxxxxxx signature
5. **AVM benchmark**: AVM ns/op vs EVM ns/op with speedup factor (typically 8–12x
   in simulation)
6. **Reactor GUI audit log**: state before/after, gas estimate, deployment receipt
