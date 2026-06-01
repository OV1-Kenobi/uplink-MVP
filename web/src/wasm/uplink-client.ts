/**
 * uplink-client.ts
 *
 * The ONLY file in the web/ tree permitted to import the wasm-bindgen bundle.
 *
 * BOUNDARY RULE: No other TS file may call `fetch`, `new WebSocket`, or
 * `new EventSource` except through functions defined in this file.
 * ESLint enforces this via the deny config in ci/eslint-deny.config.js.
 *
 * All functions here are thin TypeScript wrappers over the Rust wasm-bindgen
 * exports in `./pkg/uplink_core.js`. They add:
 * - Type safety (mapping JsValue returns to typed TS types)
 * - Error normalization (Rust errors become JS Error objects)
 * - Lazy init (wasm module is loaded once on first call)
 *
 * See BOUNDARY.md for the full contract.
 */

// Lazy-loaded wasm module handle
let _wasm: typeof import("./pkg/uplink_core.js") | null = null;

async function getWasm() {
  if (!_wasm) {
    // Dynamic import — wasm bundle is code-split by Vite
    _wasm = await import("./pkg/uplink_core.js");
    await (_wasm as any).default(); // wasm-bindgen init
  }
  return _wasm;
}

// ---------------------------------------------------------------------------
// Identity
// ---------------------------------------------------------------------------

export interface IdentityPublic {
  npub: string;
}

/** Generate a fresh random Nostr identity. Returns the public npub. */
export async function createIdentity(accountIndex = 0): Promise<string> {
  const wasm = await getWasm();
  const result = wasm.create_identity(accountIndex);
  if (typeof result !== "string") throw new Error("identity creation failed");
  return result;
}

/** Restore an identity from a BIP-39 mnemonic. Returns the public npub. */
export async function restoreIdentity(
  mnemonic: string,
  accountIndex = 0
): Promise<string> {
  const wasm = await getWasm();
  const result = wasm.restore_identity(mnemonic, accountIndex);
  if (typeof result !== "string") throw new Error("identity restore failed");
  return result;
}

/** Export the mnemonic word list for backup (one-time display). */
export async function exportMnemonicWords(): Promise<string[]> {
  const wasm = await getWasm();
  const json = wasm.export_mnemonic_words();
  return JSON.parse(json as string) as string[];
}

/** Get the npub of the currently loaded identity (null if none). */
export async function getNpub(): Promise<string | null> {
  const wasm = await getWasm();
  return wasm.get_npub() ?? null;
}

// ---------------------------------------------------------------------------
// Scheduler
// ---------------------------------------------------------------------------

export interface SplitLeg {
  leg_index: number;
  recipient_npub_hex: string;
  msats: number;
  max_fee_msats: number;
  memo?: string;
  prefer_stable_channel: boolean;
}

export interface SplitPaymentIntent {
  intent_id: string;
  stream_id: string;
  period_index: number;
  source_wallet_id: string;
  legs: SplitLeg[];
  created_at_unix: number;
}

/**
 * Advance the scheduler to `nowUnix` (Unix seconds).
 * Returns an array of payment intents that became due.
 *
 * Call this on every JS timer tick (e.g. setInterval every 60s).
 */
export async function tick(nowUnix: number): Promise<SplitPaymentIntent[]> {
  const wasm = await getWasm();
  const json = wasm.tick(BigInt(nowUnix));
  return JSON.parse(json as string) as SplitPaymentIntent[];
}

// ---------------------------------------------------------------------------
// Wallet (stubs — implemented in Phase A3/A4)
// ---------------------------------------------------------------------------

export async function walletBalance(): Promise<{
  lightning_msats: number;
  onchain_confirmed_sats: number;
  stable_channel_usd_cents?: number;
}> {
  throw new Error("Wallet surface available in Phase A3");
}

export async function walletReceiveInvoice(
  _msats: number,
  _memo: string
): Promise<string> {
  throw new Error("Wallet surface available in Phase A3");
}

export async function walletPayInvoice(
  _bolt11: string,
  _maxFeeMsats: number,
  _idempotencyKey: string
): Promise<{ preimage_hex: string; total_msats_paid: number }> {
  throw new Error("Wallet surface available in Phase A3");
}
