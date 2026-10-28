# SweepNova AI ($SWEEP) — Solana High-Velocity Savings & Automated Micro-Sweeps

[![Network](https://img.shields.io/badge/solana-Devnet%20%7C%20Mainnet-14F195.svg)](https://solana.com)
[![Framework](https://img.shields.io/badge/anchor-v0.30.1-black.svg)](https://www.anchor-lang.com)

Multi-asset high-velocity savings vaults, scheduled contributions, consent-based bounded sweeps, and maturity-controlled withdrawals on Solana.

---

## 🔑 On-Chain Instructions & Security
* `initialize_protocol`: Sets multisig governance, fee vaults, and emergency controls.
* `create_user_vault`: Generates deterministic PDA user vault seeds `[b"user-vault", owner, mint]`.
* `deposit_fixed_position`: Locks assets with timestamped maturity and integer-basis yields.
* `execute_bounded_sweep`: Executes periodic auto-sweeps constrained strictly by on-chain mandate caps and time intervals.
