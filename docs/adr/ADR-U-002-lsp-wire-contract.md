# ADR-U-002 — LSP Wire Contract (Stable-Channels + OpenAgents LSP)

## Status
Proposed — **STUB**. This ADR will be finalized once the OpenAgents LSP design is complete.
The LSP team is designing the service to match this interface.

## Date
2026-06-01

## Context
Uplink requires a Lightning Service Provider (LSP) to:
1. Open and manage JIT channels to Uplink nodes.
2. Relay BOLT peer traffic to browser-based LDK nodes (TCP is not available in browsers).
3. Execute Stable-Channels target-asset balance accounting (USD-denominated positions).
4. Credit recipient Stable-Channel balances as the execution mechanism for kind-30901 streams.

The LSP is being built specifically to complement this Uplink design.
Until the wire contract is finalized, all LSP calls are stubbed in `crates/uplink-wallet/src/lsp.rs`.

## Open Questions (to be closed before Phase A4)

1. **Channel-request format**: BOLT-spec LSP spec (LSPS0/LSPS1/LSPS2), custom JSON-RPC,
   or BOLT 12 offers?
2. **Authentication**: LNURL-auth (NIP-98 style), node-key ECDSA signed challenge, or bearer token?
3. **Stable-Channels extension**: Which feature bits does the LSP advertise?
   What is the USD target-balance message format and settlement frequency?
4. **Peer bridge transport**: Does the LSP accept WebSocket peer connections from browser nodes,
   or does Uplink delegate all Lightning to a desktop/iOS node via NWC?
5. **Stable-channel credit primitive**: When a kind-30901 period fires, does Uplink:
   (a) Pay the LSP a BOLT11 invoice, which the LSP credits to the recipient's stable-channel?
   (b) Or does the LSP orchestrate the payment itself from Uplink's balance?

## Interim decision (until closed)
- All LSP calls in `uplink-wallet::lsp` return `todo!()` errors.
- Phase A3 (host-cli) uses `ldk-node` directly against regtest without an LSP.
- Phase A4 wires the real LSP after this ADR reaches Accepted status.

## References
- `crates/uplink-wallet/src/lsp.rs` (stub implementation)
- Stable-Channels: `https://www.stablechannels.com/`
- LSPS specs: `https://github.com/BitcoinAndLightningLayerSpecs/lsp`
