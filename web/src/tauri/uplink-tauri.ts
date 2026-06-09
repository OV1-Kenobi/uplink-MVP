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

/** Decrypt and return the mnemonic word list for the one-time backup screen. */
export async function exportMnemonicWords(passphrase: string): Promise<string[]> {
  return invoke<string[]>("export_mnemonic", { passphrase });
}
