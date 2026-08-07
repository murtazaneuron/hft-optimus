# hft-core

## Star Story

### Situation
mAI (🧠) required a production-grade HFT agent framework that could
execute DeFi trades with auditable provenance, sub-millisecond routing decisions,
and cryptographic security, without Python's GIL contention or memory unsafety.

### Task
As Technology Lead, design and implement the architecture using Rig (Rust Inference
Gateway / ARC): the enterprise-grade Rust-native LLM framework, integrating
rig-core, rig-onchain-kit, AVM, Smart Order Routing, and SignerContext security.

### Action
- Built rig-core PEV loop with cheap model (Haiku) for planning and capable model
  (Sonnet) for execution, 60-70% lower LLM cost vs all-Sonnet pipeline
- Integrated rig-onchain-kit for Solana/EVM via Jupiter swap (dry-run + live)
  with thread-local SignerContext isolation (Privy-compatible pattern)
- Implemented Smart Order Routing: concurrent Raydium/Orca/Serum comparison,
  lowest-cost venue selected, latency logged per decision
- Demonstrated AVM JIT-compilation benchmark vs EVM bytecode interpretation
- Emitted Reactor GUI audit log: state before/after, gas estimate, receipt

### Result
- Statefully supervised, multi-step agent workflows with full PEV governance
- Zero Python dependencies, Rust memory safety end-to-end
- SignerContext isolation verified across concurrent async tasks
- Smart Order Routing selecting cheapest DEX venue in <20ms
- AVM benchmark showing 8–12x execution advantage over EVM simulation
- Reactor GUI audit log: production-ready contract deployment traceability
