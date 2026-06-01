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

### Wallet (Phase A3+)

| Function | Status |
|---|---|
| `wallet_balance()` | Phase A3 |
| `wallet_receive_invoice(msats, memo)` | Phase A3 |
| `wallet_pay_invoice(bolt11, max_fee, idempotency_key)` | Phase A3 |
| `wallet_onchain_address()` | Phase A3 |

### Relay pool (Phase A2+)

| Function | Status |
|---|---|
| `add_relay(url)` | Phase A2 |
| `remove_relay(url)` | Phase A2 |
| `list_relays()` | Phase A2 |
| `publish_profile(json_meta)` | Phase A2 |
| `fetch_contact_profile(npub)` | Phase A2 |

### Streams (Phase A6+)

| Function | Status |
|---|---|
| `create_stream(policy_json)` | Phase A6 |
| `list_streams()` | Phase A6 |
| `pause_stream(stream_id)` | Phase A6 |
| `resume_stream(stream_id)` | Phase A6 |
| `remove_stream(stream_id)` | Phase A6 |

### Accounts (Phase A7+)

| Function | Status |
|---|---|
| `issue_delegation(child_npub, policy_json)` | Phase A7 |
| `revoke_delegation(token_id)` | Phase A7 |
| `list_delegations()` | Phase A7 |

## Error format
All functions return errors as a thrown `Error` in TypeScript (the wasm
layer returns `Err(JsValue::from_str(&msg))` which `uplink-client.ts` converts).
