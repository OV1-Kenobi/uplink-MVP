# Uplink wasm-bindgen Surface Contract

This file is the canonical reference for the TypeScript↔Rust boundary.

## The rule

**TypeScript MUST NOT** call `fetch()`, `new WebSocket()`, `new EventSource()`,
or any other network API directly. The **only** permitted exception is
`web/src/wasm/uplink-client.ts`, which wraps the wasm-bindgen bundle.

ESLint enforces this rule via `ci/eslint-deny.config.js`.
`cargo deny` enforces the Rust side via `ci/cargo-deny.toml`.

## Exported functions

All functions are exported from `crates/uplink-core/src/ffi.rs` and wrapped
in `web/src/wasm/uplink-client.ts`.

### Identity

| Function | Arguments | Returns | Notes |
|---|---|---|---|
| `create_identity(idx, pass)` | `u32, &str` | `npub: string` | Async; generates mnemonic; persists encrypted |
| `restore_identity(phr, idx, pass)` | `&str, u32, &str` | `npub: string` | Async; restores; persists encrypted |
| `unlock_identity(pass)` | `&str` | `npub: string` | Async; loads from storage |
| `export_mnemonic_words()` | — | `string[]` | Sync; one-time; zeros after retrieval |
| `get_npub()` | — | `string | null` | Sync; public identity only |
| `add_relay(url)` | `string` | `void` | Async; adds and connects to relay |
| `fetch_profile(npub)` | `string` | `ResolvedProfile` | Async; fetches kind 0 metadata |

### Scheduler

| Function | Arguments | Returns | Notes |
|---|---|---|---|
| `tick(now_unix)` | `u64` | `SplitPaymentIntent[]` | Advance scheduler to `now_unix`; returns due intents |

### Wallet

| Function | Arguments | Returns | Notes |
|---|---|---|---|
| `init_wallet(esplora_url)` | `string` | `void` | Async; initializes LDK node |
| `get_balance()` | — | `WalletBalance` | Sync; balance snapshot |
| `get_receive_address()` | — | `string` | Sync; new on-chain address |
| `get_invoice(msats, memo)` | `u64, string` | `string` | Sync; new BOLT11 invoice |
| `pay_invoice(bolt11, fee, key)` | `string, u64, string` | `PaymentResult` | Async; pay with idempotency |

### Streams & Scheduler

| Function | Arguments | Returns | Notes |
|---|---|---|---|
| `tick(now_unix)` | `u64` | `SplitPaymentIntent[]` | Advance scheduler; returns due intents |
| `upsert_stream(id, p, amt, per, start)` | `string, string, u64, u64, u64` | `void` | Add/update stream policy |
| `remove_stream(id)` | `string` | `void` | Remove stream |
| `mark_executed(id, idx)` | `string, u64` | `void` | Mark period as paid |
| `publish_stream_declaration(...)` | `...` | `void` | Async; publish kind-30901 to Nostr |
| `create_receipt(...)` | `...` | `ReceiptResult` | Async; sign and publish kind-9901 |

### Delegation

| Function | Arguments | Returns | Notes |
|---|---|---|---|
| `create_delegation(npub, id, max, cap, exp)` | `string, string, u64, u64, u64` | `DelegationToken` | Async; issue new token |

## Error format
All functions return errors as a thrown `Error` in TypeScript (the wasm
layer returns `Err(JsValue::from_str(&msg))` which `uplink-client.ts` converts).
