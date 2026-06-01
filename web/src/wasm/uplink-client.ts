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
export async function createIdentity(passphrase: string, accountIndex = 0): Promise<string> {
  const wasm = await getWasm();
  const result = await wasm.create_identity(accountIndex, passphrase);
  return result as string;
}

/** Restore an identity from a BIP-39 mnemonic. Returns the public npub. */
export async function restoreIdentity(
  mnemonic: string,
  passphrase: string,
  accountIndex = 0
): Promise<string> {
  const wasm = await getWasm();
  const result = await wasm.restore_identity(mnemonic, accountIndex, passphrase);
  return result as string;
}

/** Unlock an identity from storage using the passphrase. Returns npub. */
export async function unlockIdentity(passphrase: string): Promise<string> {
  const wasm = await getWasm();
  const result = await wasm.unlock_identity(passphrase);
  return result as string;
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
// Nostr (Relays & Profiles)
// ---------------------------------------------------------------------------

export interface ResolvedProfile {
  npub: string;
  name?: string;
  display_name?: string;
  about?: string;
  picture?: string;
  nip05?: string;
  nip05_verified: boolean;
}

/** Add a relay to the pool. */
export async function addRelay(url: string): Promise<void> {
  const wasm = await getWasm();
  await wasm.add_relay(url);
}

/** Fetch a profile by npub. */
export async function fetchProfile(npub: string): Promise<ResolvedProfile> {
  const wasm = await getWasm();
  const json = await wasm.fetch_profile(npub);
  return JSON.parse(json as string) as ResolvedProfile;
}

/** Publish a kind-30901 stream declaration to the relay pool. */
export async function publishStreamDeclaration(
  streamId: string,
  recipientNpub: string,
  msatsPerPeriod: number,
  periodSeconds: number,
  startAtUnix: number
): Promise<void> {
  const wasm = await getWasm();
  await wasm.publish_stream_declaration(
    streamId,
    recipientNpub,
    BigInt(msatsPerPeriod),
    BigInt(periodSeconds),
    BigInt(startAtUnix)
  );
}

/** Publish a kind-9901 receipt event. */
export async function publishReceipt(
  streamId: string,
  streamEventId: string,
  recipientNpub: string,
  periodIndex: number,
  msatsPaid: number,
  preimageHex: string
): Promise<void> {
  const wasm = await getWasm();
  await wasm.publish_receipt(
    streamId,
    streamEventId,
    recipientNpub,
    BigInt(periodIndex),
    BigInt(msatsPaid),
    preimageHex
  );
}


// ---------------------------------------------------------------------------
// Wallet
// ---------------------------------------------------------------------------

export interface WalletBalance {
  lightning_msats: number;
  onchain_confirmed_sats: number;
  stable_channel_usd_cents?: number;
}

export interface PaymentResult {
  preimage_hex: string;
  total_msats_paid: number;
  idempotency_key: string;
}

/** Initialize the Wasm LDK wallet. */
export async function initWallet(esploraUrl: string): Promise<void> {
  const wasm = await getWasm();
  await wasm.init_wallet(esploraUrl);
}

/** Get current balance. */
export async function getBalance(): Promise<WalletBalance> {
  const wasm = await getWasm();
  const json = wasm.get_balance();
  return JSON.parse(json as string) as WalletBalance;
}

/** Get a new on-chain receive address. */
export async function getReceiveAddress(): Promise<string> {
  const wasm = await getWasm();
  const result = wasm.get_receive_address();
  return result as string;
}

/** Generate a BOLT11 invoice. */
export async function getInvoice(msats: number, memo: string): Promise<string> {
  const wasm = await getWasm();
  const result = wasm.get_invoice(BigInt(msats), memo);
  return result as string;
}

/** Pay a BOLT11 invoice. */
export async function payInvoice(
  bolt11: string,
  maxFeeMsats: number,
  idempotencyKey: string
): Promise<PaymentResult> {
  const wasm = await getWasm();
  const json = await wasm.pay_invoice(bolt11, BigInt(maxFeeMsats), idempotencyKey);
  return JSON.parse(json as string) as PaymentResult;
}

// ---------------------------------------------------------------------------
// Receipts (Phase A5)
// ---------------------------------------------------------------------------

export interface ReceiptResult {
  event_id: string;
  receipt_hash: string;
}

/**
 * Build, sign, and publish a kind-9901 stable-stream receipt event.
 * Returns the Nostr event ID and the canonical SHA-256 receipt hash.
 */
export async function createReceipt(params: {
  streamId: string;
  streamEventId: string;
  recipientNpub: string;
  periodIndex: number;
  msatsPaid: number;
  preimageHex: string;
  paidAtUnix: number;
}): Promise<ReceiptResult> {
  const wasm = await getWasm();
  const json = await wasm.create_receipt(
    params.streamId,
    params.streamEventId,
    params.recipientNpub,
    BigInt(params.periodIndex),
    BigInt(params.msatsPaid),
    params.preimageHex,
    BigInt(params.paidAtUnix),
  );
  return JSON.parse(json as string) as ReceiptResult;
}
