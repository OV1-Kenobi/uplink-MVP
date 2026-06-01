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
// Thread-local state (Phase A1 will persist these to IndexedDB)
// ---------------------------------------------------------------------------

#[cfg(feature = "wasm")]
use std::cell::RefCell;
use uplink_identity::UplinkIdentity;
use uplink_scheduler::Scheduler;

#[cfg(feature = "wasm")]
thread_local! {
    static IDENTITY: RefCell<Option<UplinkIdentity>> = RefCell::new(None);
    static SCHEDULER: RefCell<Scheduler> = RefCell::new(Scheduler::new(vec![]));
}
