# hft-core

## HIGH-LEVEL ARCHITECTURE DIAGRAM

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    hft-core                                   │
│                    Optimal HFT Platform  (Rig / ARC)                    │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
          ┌─────────────────────────▼─────────────────────────┐
          │            CLI Entry Point  (main.rs)             │
          │   --mode [pev|sor|signer|reactor|full]            │
          └───────────────┬───────────────────────────────────┘
                          │
     ┌────────────────────┴─────────────────────────────┐
     │                                                  │
     ▼                                                  ▼
┌─────────────┐                              ┌──────────────────┐
│  PEV Loop   │                              │  Smart Order     │
│  (pev.rs)   │                              │  Routing (sor.rs)│
│             │                              │                  │
│ 1. PLAN     │                              │ Raydium    ──┐   │
│  rig-core   │                              │ Orca       ──┼──▶│ SOR
│  Haiku LLM  │                              │ Serum      ──┘   │ Decision
│             │                              └──────────────────┘
│ 2. EXECUTE  │                                       │
│  rig-core   │                                       ▼
│  Sonnet LLM │               ┌─────────────────────────────────┐
│  Tool calls │               │  rig-onchain-kit  (onchain.rs)  │
│             │               │                                 │
│ 3. VERIFY   │               │  SignerContext::with_signer()──▶│
│  Score 0–1  │               │  Jupiter swap (dry-run)         │
│  Pass ≥0.80 │               │  Raydium pool lookup            │
│             │               │  Balance query                  │
└─────────────┘               │  Privy wallet abstraction       │
       │                      └─────────────────────────────────┘
       │                                       │
       ▼                                       ▼
┌─────────────────────────────────────────────────────────────────┐
│              AVM Execution Layer  (avm.rs)                      │
│  JIT-compiled execution simulation · EVM comparison benchmark   │
│  Reactor GUI Audit Log: state before/after · gas estimate       │
└─────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
              ┌───────────────────────────────────┐
              │  Structured Execution Log         │
              │  (JSON + terminal output)         │
              │  Defence trail │
              └───────────────────────────────────┘
```
