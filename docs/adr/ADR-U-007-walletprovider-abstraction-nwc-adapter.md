# ADR-U-007 — WalletProvider Abstraction + NWC Adapter

## Status
Accepted

## Date
2026-06-09

## Context
Uplink's business logic (split payments, streaming intervals, zaps) currently reaches
the wallet through `WalletExecutor` — a small, **synchronous** trait with exactly one
real implementation (`NativeLdkWallet`, behind the `native` feature) and a non-functional
`WasmLdkWallet`. Three problems block the pivot (ADR-U-006):

1. **Tight coupling to LDK.** The remote-wallet direction needs the app to drive an
   *external* wallet (the user's existing node/wallet) over **NIP-47 Nostr Wallet
   Connect**, not only an embedded LDK node. The sync, LDK-shaped trait cannot model a
   request/response protocol over a relay.
2. **`todo!()` panics in production paths.** `uplink-nostr/zap.rs`, `uplink-cashu`, and
   `uplink-wallet/lsp.rs` contain `todo!()` macros that would panic if reached — a direct
   violation of the AGENTS.md "no `todo!()` in production paths" invariant.
3. **No recipient resolution.** There is no way to turn a NIP-05 / Lightning-address /
   LNURL / npub into a payable BOLT11 invoice; `zap.rs` only has stubs.

This ADR governs `crates/uplink-wallet` (the trait + LDK adapter) and the NWC adapter +
recipient resolver in `crates/uplink-nostr`. It is the abstraction that later isolates
the wallet-engine swap in Phase 9 (LDK / MoneyDevKit) from every higher layer.

## Decision

### 1. `WalletProvider` — the async, capability-described wallet surface
A new **async** trait (`#[async_trait]`) modeled on NIP-47, in
`uplink-wallet/src/provider.rs`:

```
get_info()           -> WalletInfo          // node pubkey, network, methods, caps
get_balance()        -> WalletBalance        // reuses the existing balance shape
make_invoice(msats, description) -> Invoice
pay_invoice(bolt11, max_fee_msats: Option<u64>) -> PaymentResult
lookup_invoice(payment_hash)      -> InvoiceStatus
list_transactions(ListTxParams)   -> Vec<Transaction>
is_available()       -> bool                 // sync; cheap liveness hint
get_capabilities()   -> WalletCapabilities   // sync; declared feature bits
```

`WalletExecutor` is **retained** (host-cli + wasm ffi still use it); `WalletProvider` is
the new surface business logic targets. Value types (`WalletInfo`, `Invoice`,
`InvoiceStatus`, `Transaction`, `TxKind`, `ListTxParams`, `WalletCapabilities`,
`ProviderError`) are `serde`-friendly and platform-agnostic.

### 2. Two adapters
- **`ExecutorProvider<W: WalletExecutor>`** (uplink-wallet) — wraps any `WalletExecutor`
  (today: `NativeLdkWallet`) into a `WalletProvider`. Methods the executor cannot serve
  (`lookup_invoice`, `list_transactions`) return `ProviderError::Unsupported`. Carries a
  `spend_capable` flag (see §4).
- **`NwcProvider`** (uplink-nostr) — a NIP-47 client. It parses a
  `nostr+walletconnect://` URI, encodes kind-23194 requests (NIP-04-encrypted JSON),
  awaits the kind-23195 response, and decodes results. All relay I/O is behind a
  **`Nip47Transport`** shim trait (`async fn request(req_event) -> response_event`), so
  the protocol layer is unit-testable without a live relay (mirrors the Phase 4
  `Transceive` shim pattern).

### 3. Recipient resolver — resolves the `zap.rs` `todo!()`
`uplink-nostr/src/recipient.rs` adds `RecipientAddress`
(`Npub` / `LightningAddress` / `Lnurl` / `Bolt11`) with a pure, fully-tested
`parse()` and `lnurlp_url()` (NIP-05/lightning-address → `/.well-known/lnurlp/<user>`;
`lnurl1…` bech32 → URL). Invoice fetching is implemented in terms of an **`LnurlClient`**
HTTP shim (`async fn fetch(url) -> body`), so `resolve_invoice()` is testable with a mock
and carries no `todo!()`. `zap.rs::resolve_lightning_address` becomes a real kind-0
(`lud16`/`lud06`) parser; `build_zap_invoice` delegates to the resolver via an
`LnurlClient`.

### 4. Two-credential split scaffolding
`WalletCapabilities` exposes `spend_capable: bool`. A receive-only credential (Phase 5
identity service) advertises `spend_capable = false`; higher layers must check it before
attempting `pay_invoice`. This ADR only scaffolds the flag and the check point.

### 5. Stub cleanup — no `todo!()` in production paths
The remaining `todo!()` macros are replaced by structured, honest errors rather than
panics: `uplink-wallet/lsp.rs` returns `WalletError::Lsp("… not yet available …")`;
`uplink-cashu::send_nutzap` returns a new `NutzapError::NotEnabled`. The real
implementations land in their owning phases (LSP wire contract ADR-U-002; nutzap
fallback later). `grep -rn "todo!\|unimplemented!"` over `crates/` + `host-cli/` returns
nothing after this change.

## Consequences
- **Positive:** business logic depends on a stable, async, capability-described wallet
  surface; an external NIP-47 wallet and the embedded LDK node are interchangeable;
  recipient resolution is real and tested; the no-`todo!()` invariant holds; the Phase 9
  engine swap touches only an adapter.
- **Negative / deferred:** `NwcProvider` ships with a shim transport — the live relay
  transport and `LnurlClient` HTTP impl are wired at the platform boundary (Tauri native
  / wasm) where the network client lives; this ADR provides the protocol + mocks. The LSP
  and nutzap paths are gated (return errors), not yet functional.
- **Migration:** additive. `WalletExecutor` and its callers are untouched; `WalletProvider`
  is new surface. New value types are `serde(default)`-friendly for forward compatibility.

## References
- `docs/plans/uplink-pivot.md` — Phase 3
- `docs/adr/ADR-U-006-platform-pivot-tauri-foss-distribution.md` — platform pivot
- `docs/adr/ADR-U-002-lsp-wire-contract.md` — LSP wire contract (still pending)
- NIP-47 (Nostr Wallet Connect), NIP-57 (zaps), LNURL-pay / LUD-06 / LUD-16
- `AGENTS.md` — no-`todo!()` + idempotency invariants
