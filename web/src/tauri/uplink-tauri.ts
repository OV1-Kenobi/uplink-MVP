// Tauri native boundary client (see BOUNDARY.md, ADR-U-006).
//
// Under the Tauri shell, the UI reaches the native Rust core through these
// `invoke()` wrappers instead of the wasm bundle. Custody never returns to the
// UI: identity calls resolve to the public npub only. This is the native
// counterpart to `web/src/wasm/uplink-client.ts`.

import { invoke } from "@tauri-apps/api/core";

export interface IdentityInfo {
  npub: string;
  account: number;
}

/** True when running inside the Tauri shell (vs. the plain web/Netlify build). */
export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

/** Crate version — proves the native command bridge before any identity exists. */
export async function appVersion(): Promise<string> {
  return invoke<string>("app_version");
}

/** Generate a new identity; persists encrypted natively, returns the npub. */
export async function createIdentity(
  passphrase: string,
  account = 0,
): Promise<string> {
  return invoke<string>("create_identity", { passphrase, account });
}

/** Restore an identity from a mnemonic; persists natively, returns the npub. */
export async function restoreIdentity(
  mnemonic: string,
  passphrase: string,
  account = 0,
): Promise<string> {
  return invoke<string>("restore_identity", { mnemonic, passphrase, account });
}

/** Load the persisted identity descriptor, or null if none exists. */
export async function currentIdentity(
  passphrase: string,
): Promise<IdentityInfo | null> {
  return invoke<IdentityInfo | null>("current_identity", { passphrase });
}

/** Passphrase-free probe: is an identity already provisioned on this device? */
export async function hasIdentity(): Promise<boolean> {
  return invoke<boolean>("has_identity");
}

/** Decrypt and return the mnemonic word list for the one-time backup screen.
 *  Uses the unlocked session passphrase held natively (single-unlock model). */
export async function exportMnemonicWords(): Promise<string[]> {
  return invoke<string[]>("export_mnemonic");
}

/** Clear the provisioned identity from the native store ("Reset app"). */
export async function resetIdentity(): Promise<void> {
  await invoke("reset_identity");
}

/** Lock the session: clear the in-memory passphrase held natively (sign-out / lock). */
export async function lockSession(): Promise<void> {
  await invoke("lock_session");
}

// ── Phase 5a — external credentials + relay set (ADR-U-010) ──────────────────
// Bearer secrets (NWC URI, LNC pairing phrase) never return to the UI: every call
// below resolves to the redacted, non-secret `CredentialMeta`. These run against the
// unlocked session passphrase held natively (single-unlock model) — no passphrase
// crosses the boundary per call.

export type CredentialKind = "lightning_address" | "nip05" | "npub" | "nwc" | "lnc";

/** Redacted, non-secret descriptor of a linked credential. */
export interface CredentialMeta {
  kind: CredentialKind;
  label: string;
  receive_capable: boolean;
  spend_capable: boolean;
  added_at_unix: number;
}

/** Link an NWC connection string (receive + spend). */
export async function connectNwc(uri: string): Promise<CredentialMeta> {
  return invoke<CredentialMeta>("connect_nwc", { uri });
}

/** Link a Lightning Node Connect 10-word pairing phrase (spend; LND-direct, gated). */
export async function connectLnc(pairingPhrase: string): Promise<CredentialMeta> {
  return invoke<CredentialMeta>("connect_lnc", { pairingPhrase });
}

/** Set the user's own Lightning Address (primary receive path). */
export async function setLightningAddress(address: string): Promise<CredentialMeta> {
  return invoke<CredentialMeta>("set_lightning_address", { address });
}

/** Link an existing Nostr identity (npub or NIP-05). */
export async function linkIdentity(
  kind: "npub" | "nip05",
  value: string,
): Promise<CredentialMeta> {
  return invoke<CredentialMeta>("link_identity", { kind, value });
}

/** List linked credentials as redacted descriptors. */
export async function listCredentials(): Promise<CredentialMeta[]> {
  return invoke<CredentialMeta[]>("list_credentials");
}

/** Remove a linked credential by its kind. */
export async function disconnectCredential(kind: CredentialKind): Promise<void> {
  await invoke("disconnect_credential", { kind });
}

/** Persisted relay set, or null if the user has never customized it. */
export async function getRelays(): Promise<string[] | null> {
  return invoke<string[] | null>("get_relays");
}

/** Persist the user's relay set. */
export async function setRelays(relays: string[]): Promise<void> {
  await invoke("set_relays", { relays });
}
