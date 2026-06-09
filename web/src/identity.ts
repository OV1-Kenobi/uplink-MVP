/**
 * identity.ts — runtime-dispatching identity facade.
 *
 * Components call these functions; this module routes each call to the native
 * Tauri bridge (`tauri/uplink-tauri.ts`) when running inside the Tauri shell,
 * or to the wasm bundle wrapper (`wasm/uplink-client.ts`) otherwise. Both
 * targets uphold the same custody invariant (see BOUNDARY.md, ADR-U-006):
 * the mnemonic reaches the UI only through the one-time backup export.
 *
 * This file performs no network I/O of its own — it only dispatches to the two
 * established boundary modules.
 */

import { isTauri } from "./tauri/uplink-tauri.ts";
import * as native from "./tauri/uplink-tauri.ts";
import * as wasm from "./wasm/uplink-client.ts";

/** Generate a new identity; returns the public npub. */
export async function createIdentity(
  passphrase: string,
  accountIndex = 0,
): Promise<string> {
  return isTauri()
    ? native.createIdentity(passphrase, accountIndex)
    : wasm.createIdentity(passphrase, accountIndex);
}

/** Restore an identity from a BIP-39 mnemonic; returns the public npub. */
export async function restoreIdentity(
  mnemonic: string,
  passphrase: string,
  accountIndex = 0,
): Promise<string> {
  return isTauri()
    ? native.restoreIdentity(mnemonic, passphrase, accountIndex)
    : wasm.restoreIdentity(mnemonic, passphrase, accountIndex);
}

/** Unlock the persisted identity with the passphrase; returns the public npub. */
export async function unlockIdentity(passphrase: string): Promise<string> {
  if (isTauri()) {
    const info = await native.currentIdentity(passphrase);
    if (!info) {
      throw new Error("No identity provisioned on this device");
    }
    return info.npub;
  }
  return wasm.unlockIdentity(passphrase);
}

/** Export the mnemonic word list for the one-time backup screen.
 *  Uses the session unlocked at create/restore/unlock — no passphrase needed. */
export async function exportMnemonicWords(): Promise<string[]> {
  return isTauri()
    ? native.exportMnemonicWords()
    : wasm.exportMnemonicWords();
}

/**
 * Lock the session: clear the in-memory passphrase held by the native layer.
 *
 * No-op on the plain browser build (the wasm identity is held only until reload);
 * under Tauri this drops the KEK so subsequent connection use requires a fresh unlock.
 */
export async function lockSession(): Promise<void> {
  if (isTauri()) await native.lockSession();
}

/** Probe whether an identity is already provisioned on this device. */
export async function hasIdentity(): Promise<boolean> {
  if (isTauri()) {
    return native.hasIdentity();
  }
  return (
    typeof localStorage !== "undefined" &&
    localStorage.getItem("identity_mnemonic") !== null
  );
}

/**
 * Clear the provisioned identity on this device ("Reset app").
 *
 * Under Tauri this clears the native `sled` store; on the wasm/browser target
 * it clears `localStorage`. Routed through the facade so the onboarding reset
 * works on both targets (previously `localStorage.clear()` left the native
 * store intact).
 */
export async function resetIdentity(): Promise<void> {
  if (isTauri()) {
    await native.resetIdentity();
    return;
  }
  if (typeof localStorage !== "undefined") {
    localStorage.clear();
  }
}

// ── Phase 5a — external credentials + relay set (ADR-U-010) ──────────────────
// These are native (Tauri app) features: bearer secrets are held and encrypted in the
// Rust layer. On the plain browser build they are unavailable, so the facade returns
// safe fallbacks (empty list / null relays) or throws for explicit link actions.

export type { CredentialKind, CredentialMeta } from "./tauri/uplink-tauri.ts";
import type { CredentialKind, CredentialMeta } from "./tauri/uplink-tauri.ts";

function appOnly(): never {
  throw new Error("Wallet connections are only available in the Uplink app");
}

/** Link an NWC connection string (receive + spend). */
export async function connectNwc(uri: string): Promise<CredentialMeta> {
  return isTauri() ? native.connectNwc(uri) : appOnly();
}

/** Link a Lightning Node Connect pairing phrase (spend; LND-direct, gated). */
export async function connectLnc(pairingPhrase: string): Promise<CredentialMeta> {
  return isTauri() ? native.connectLnc(pairingPhrase) : appOnly();
}

/** Set the user's own Lightning Address (primary receive path). */
export async function setLightningAddress(address: string): Promise<CredentialMeta> {
  return isTauri() ? native.setLightningAddress(address) : appOnly();
}

/** Link an existing Nostr identity (npub or NIP-05). */
export async function linkIdentity(
  kind: "npub" | "nip05",
  value: string,
): Promise<CredentialMeta> {
  return isTauri() ? native.linkIdentity(kind, value) : appOnly();
}

/** List linked credentials as redacted descriptors (empty on the browser build). */
export async function listCredentials(): Promise<CredentialMeta[]> {
  return isTauri() ? native.listCredentials() : [];
}

/** Remove a linked credential by its kind. */
export async function disconnectCredential(kind: CredentialKind): Promise<void> {
  if (isTauri()) await native.disconnectCredential(kind);
}

/** Persisted relay set, or null if never customized (always null on the browser build). */
export async function getRelays(): Promise<string[] | null> {
  return isTauri() ? native.getRelays() : null;
}

/** Persist the user's relay set (no-op on the browser build). */
export async function setRelays(relays: string[]): Promise<void> {
  if (isTauri()) await native.setRelays(relays);
}
