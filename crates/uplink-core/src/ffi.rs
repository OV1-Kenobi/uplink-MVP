//! wasm-bindgen surface — all exported functions documented in BOUNDARY.md.
//!
//! ## Conventions
//! - All functions return `Result<JsValue, JsValue>` where the Err is a JSON
//!   object `{ "error": "<message>" }`.
//! - Secret material (mnemonic, seeds, signing keys) is NEVER returned as JsValue.
//!   The identity is held in a thread-local RefCell; JS only sees the npub string.
//! - All serialized return values use `serde_json` to produce JSON-compatible JsValues.

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

use nostr::prelude::*;
use std::cell::RefCell;
use std::sync::Arc;
use uplink_identity::UplinkIdentity;
use uplink_nostr::relay::RelayPool;
use uplink_scheduler::Scheduler;


// ---------------------------------------------------------------------------
// Identity surface
// ---------------------------------------------------------------------------

/// Generate a new random BIP-39 identity and return only the public npub.
///
/// The mnemonic is held in memory only. The caller MUST immediately call
/// `export_mnemonic_words()` to display the backup phrase to the user.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn create_identity(account_index: u32, passphrase: &str) -> Result<JsValue, JsValue> {
    use uplink_identity::UplinkIdentity;
    use uplink_storage::{KvStore, PlatformStore};


    let id = UplinkIdentity::generate(account_index)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Persist
    let store = PlatformStore::open(passphrase).map_err(|e| JsValue::from_str(&e.to_string()))?;
    store.put("identity_mnemonic", id.mnemonic_phrase().as_bytes()).await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    store.put("identity_account", &account_index.to_be_bytes()).await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Store in thread-local
    IDENTITY.with(|cell| *cell.borrow_mut() = Some(id.clone()));
    Ok(JsValue::from_str(&id.npub()))
}

/// Restore an identity from a mnemonic phrase. Returns the npub.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn restore_identity(mnemonic: &str, account_index: u32, passphrase: &str) -> Result<JsValue, JsValue> {
    use uplink_identity::UplinkIdentity;
    use uplink_storage::{KvStore, PlatformStore};

    let id = UplinkIdentity::from_mnemonic_str(mnemonic, account_index)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    // Persist
    let store = PlatformStore::open(passphrase).map_err(|e| JsValue::from_str(&e.to_string()))?;
    store.put("identity_mnemonic", id.mnemonic_phrase().as_bytes()).await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;
    store.put("identity_account", &account_index.to_be_bytes()).await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    IDENTITY.with(|cell| *cell.borrow_mut() = Some(id.clone()));
    Ok(JsValue::from_str(&id.npub()))
}

/// Unlock an existing identity from storage.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn unlock_identity(passphrase: &str) -> Result<JsValue, JsValue> {
    use uplink_identity::UplinkIdentity;
    use uplink_storage::{KvStore, PlatformStore};

    let store = PlatformStore::open(passphrase).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let mnemonic_bytes = store.get("identity_mnemonic").await
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .ok_or_else(|| JsValue::from_str("no identity found"))?;
    let mnemonic = String::from_utf8(mnemonic_bytes).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let account_bytes = store.get("identity_account").await
        .map_err(|e| JsValue::from_str(&e.to_string()))?
        .unwrap_or_else(|| 0u32.to_be_bytes().to_vec());
    let account = u32::from_be_bytes(account_bytes.try_into().unwrap_or([0u8; 4]));

    let id = UplinkIdentity::from_mnemonic_str(&mnemonic, account)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    IDENTITY.with(|cell| *cell.borrow_mut() = Some(id.clone()));
    Ok(JsValue::from_str(&id.npub()))
}

/// Export the mnemonic word list for backup display (one-time; zeroed after call).
///
/// Returns JSON array of 12 or 24 strings.
/// After this call, the user must confirm backup before any funds are received.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn export_mnemonic_words() -> Result<JsValue, JsValue> {
    IDENTITY.with(|cell| {
        let borrow = cell.borrow();
        match borrow.as_ref() {
            None => Err(JsValue::from_str("no identity loaded")),
            Some(id) => {
                let words = id.mnemonic_words();
                serde_json::to_value(&words)
                    .map(|v| JsValue::from_str(&v.to_string()))
                    .map_err(|e| JsValue::from_str(&e.to_string()))
            }
        }
    })
}

/// Return the current identity's npub (bech32).
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_npub() -> Option<String> {
    IDENTITY.with(|cell| cell.borrow().as_ref().map(|id| id.npub()))
}

// ---------------------------------------------------------------------------
// Relay surface
// ---------------------------------------------------------------------------

/// Add a relay to the pool.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn add_relay(url: String) -> Result<(), JsValue> {
    use uplink_nostr::relay::RelayPool;

    let keys = IDENTITY.with(|cell| {
        cell.borrow().as_ref().map(|id| id.nostr_keys.clone())
    }).ok_or_else(|| JsValue::from_str("no identity loaded"))?;

    let mut pool_opt = RELAY_POOL.with(|cell| cell.borrow().clone());
    if pool_opt.is_none() {
        let config = uplink_nostr::relay::RelayConfig::default();
        let pool = RelayPool::new(config, &keys);
        pool.connect().await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        pool_opt = Some(pool);
    }

    if let Some(mut pool) = pool_opt {
        pool.add_relay(url).await.map_err(|e| JsValue::from_str(&e.to_string()))?;
        RELAY_POOL.with(|cell| *cell.borrow_mut() = Some(pool));
    }
    Ok(())
}

/// Fetch a profile by npub.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn fetch_profile(npub: &str) -> Result<JsValue, JsValue> {
    let pk = PublicKey::from_bech32(npub).map_err(|e| JsValue::from_str(&e.to_string()))?;

    let pool = RELAY_POOL.with(|cell| cell.borrow().clone())
        .ok_or_else(|| JsValue::from_str("relay pool not initialized (call add_relay or connect first)"))?;

    let profile = pool.resolve_profile(pk).await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_value(&profile)
        .map(|v| JsValue::from_str(&v.to_string()))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

// ---------------------------------------------------------------------------
// Scheduler surface
// ---------------------------------------------------------------------------

/// Advance the scheduler to `now_unix` (Unix seconds).
///
/// Returns JSON array of `SplitPaymentIntent`s that became due.
/// Call this on every JS `setInterval` tick.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn tick(now_unix: u64) -> Result<JsValue, JsValue> {
    SCHEDULER.with(|cell| {
        let sched = cell.borrow();
        let intents = sched.tick(now_unix);
        serde_json::to_value(&intents)
            .map(|v| JsValue::from_str(&v.to_string()))
            .map_err(|e| JsValue::from_str(&e.to_string()))
    })
}

// ---------------------------------------------------------------------------
// Wallet surface
// ---------------------------------------------------------------------------

/// Initialize the Wasm LDK wallet.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn init_wallet(esplora_url: String) -> Result<(), JsValue> {
    use uplink_wallet::wasm::WasmLdkWallet;

    let id = IDENTITY.with(|cell| cell.borrow().clone())
        .ok_or_else(|| JsValue::from_str("no identity loaded"))?;

    let wallet = WasmLdkWallet::new(&id, &esplora_url).await
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    WALLET.with(|cell| *cell.borrow_mut() = Some(Arc::new(wallet)));
    Ok(())
}

/// Get the current balance snapshot.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_balance() -> Result<JsValue, JsValue> {
    let wallet = WALLET.with(|cell| cell.borrow().clone())
        .ok_or_else(|| JsValue::from_str("wallet not initialized"))?;

    let balance = wallet.balance().map_err(|e| JsValue::from_str(&e.to_string()))?;
    serde_json::to_value(&balance)
        .map(|v| JsValue::from_str(&v.to_string()))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}

/// Get a new on-chain receive address.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_receive_address() -> Result<JsValue, JsValue> {
    let wallet = WALLET.with(|cell| cell.borrow().clone())
        .ok_or_else(|| JsValue::from_str("wallet not initialized"))?;

    let addr = wallet.receive_onchain_address().map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(JsValue::from_str(&addr))
}

/// Generate a BOLT11 invoice.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub fn get_invoice(msats: u64, memo: String) -> Result<JsValue, JsValue> {
    let wallet = WALLET.with(|cell| cell.borrow().clone())
        .ok_or_else(|| JsValue::from_str("wallet not initialized"))?;

    let invoice = wallet.receive_invoice(msats, &memo).map_err(|e| JsValue::from_str(&e.to_string()))?;
    Ok(JsValue::from_str(&invoice))
}

/// Pay a BOLT11 invoice.
#[cfg(feature = "wasm")]
#[wasm_bindgen]
pub async fn pay_invoice(bolt11: String, max_fee_msats: u64, idempotency_key: String) -> Result<JsValue, JsValue> {
    let wallet = WALLET.with(|cell| cell.borrow().clone())
        .ok_or_else(|| JsValue::from_str("wallet not initialized"))?;

    let result = wallet.pay_invoice(&bolt11, max_fee_msats, &idempotency_key)
        .map_err(|e| JsValue::from_str(&e.to_string()))?;

    serde_json::to_value(&result)
        .map(|v| JsValue::from_str(&v.to_string()))
        .map_err(|e| JsValue::from_str(&e.to_string()))
}


// ---------------------------------------------------------------------------
// Thread-local state
// ---------------------------------------------------------------------------

#[cfg(feature = "wasm")]
thread_local! {
    static IDENTITY: RefCell<Option<UplinkIdentity>> = RefCell::new(None);
    static RELAY_POOL: RefCell<Option<RelayPool>> = RefCell::new(None);
    static SCHEDULER: RefCell<Scheduler> = RefCell::new(Scheduler::new(vec![]));
    static WALLET: RefCell<Option<Arc<dyn uplink_wallet::WalletExecutor>>> = RefCell::new(None);
}
