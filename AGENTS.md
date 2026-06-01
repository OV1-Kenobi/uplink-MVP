# Uplink — Agent Contract

## Quick orientation
- Architecture decisions: `docs/adr/README.md`
- wasm boundary contract: `BOUNDARY.md`
- Build plan: `README.md` (phase map)

## Mandatory pre-coding gate
Before writing or modifying any code:
1. Read `docs/adr/README.md` and identify the ADR(s) governing the surface you're touching.
2. State which ADR(s) apply and what constraints they impose.
3. If the proposed change violates an ADR, redesign before editing files.

## Authority hierarchy
- If docs conflict with code: code wins.
- If ADRs conflict with a proposed change: stop and resolve with the human first.
- `BOUNDARY.md` is non-negotiable: TS never touches the network outside `src/wasm/uplink-client.ts`.

## Repository structure
```
crates/uplink-identity/   BIP-39 → NIP-06 + LDK key derivation
crates/uplink-wallet/     Lightning + on-chain wallet (WalletExecutor trait)
crates/uplink-nostr/      Relay pool, receipt events, delegation, zaps
crates/uplink-accounts/   User/Wallet/Extension + split-payment model
crates/uplink-scheduler/  Tick-driven recurring payment scheduler
crates/uplink-receipts/   Canonical receipt format (SHA-256 canonicalization)
crates/uplink-storage/    Encrypted KvStore (IndexedDB / sled)
crates/uplink-cashu/      Cashu eCash fallback (Phase A5)
crates/uplink-core/       wasm-bindgen surface (ffi.rs = the boundary)
host-cli/                 Native binary for dev + CI regtest
web/                      React + Vite PWA shell
ci/                       ESLint deny + cargo-deny configs
docs/adr/                 Architecture decision records
```

## Build commands
```bash
# Check all Rust (native)
cargo check --workspace

# Run all tests
cargo test --workspace

# Build wasm bundle
cd web && npm run wasm:build

# Check TS
cd web && npm run typecheck

# Run ESLint boundary enforcement
cd web && npx eslint src --config ../ci/eslint-deny.config.js --max-warnings 0

# Run cargo-deny
cargo deny check
```

## Engineering invariants
- Wallet custody: mnemonic, LDK seed, NIP-60 wallet keys — NEVER cross the wasm boundary.
- Idempotency: every payment is keyed by `(intent_id, leg_index)`. Re-submitting the same key must return the original result without re-paying.
- Receipt hash: `uplink-receipts::PaymentAttemptReceipt::canonical_hash()` — do not change this format without bumping the known-answer test.
- No `todo!()` stubs in the scheduler or idempotency logic in production paths.

## Git hygiene
- Commit only files changed for the requested task.
- Do not modify OA monorepo files (this repo has no path deps to OA crates).
- Keep commits atomic per phase.
