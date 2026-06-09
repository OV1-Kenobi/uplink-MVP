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

/** Export the mnemonic word list for the one-time backup screen. */
export async function exportMnemonicWords(passphrase: string): Promise<string[]> {
  return isTauri()
    ? native.exportMnemonicWords(passphrase)
    : wasm.exportMnemonicWords();
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
