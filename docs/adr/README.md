# Uplink — Architecture Decision Records

All significant design decisions are documented here before implementation begins.

| ADR | Title | Status |
|---|---|---|
| [ADR-U-001](ADR-U-001-ldk-seed-derivation.md) | LDK Seed Derivation Path | Accepted |
| [ADR-U-002](ADR-U-002-lsp-wire-contract.md) | LSP Wire Contract (Stub) | Proposed |
| [ADR-U-003](ADR-U-003-receipt-event-kind.md) | Receipt Event Kinds (30901, 9901, 9902, 9903) | Accepted |
| [ADR-U-004](ADR-U-004-delegation-token-format.md) | Delegation Token Format | Accepted |
| [ADR-U-005](ADR-U-005-key-recovery-otp-nostr.md) | Key Recovery via Nostr OTP | Accepted |
| [ADR-U-006](ADR-U-006-platform-pivot-tauri-foss-distribution.md) | Platform Pivot to Tauri v2 + FOSS / De-Googled Distribution | Accepted |

## ADR format
Follow the template implicitly used by the above records:
Status / Date / Context / Decision / Consequences / References.

## ADR numbering
Prefix all Uplink-local ADRs with `ADR-U-NNN`. These are distinct from the OA
monorepo's `ADR-NNNN-*` series. The Deliverable B integration PR will add
cross-references between the two ADR indexes.
