//! Native Tauri commands — the Tauri ⇄ UI boundary (see BOUNDARY.md, ADR-U-006).
//!
//! These wrap the same native operations `host-cli` uses. Wallet custody
//! (mnemonic, seeds, signing keys) never crosses back to the UI: identity
//! commands return only the public npub. Persistence uses the native encrypted
//! `PlatformStore` (sled) under the app data dir.

use serde::Serialize;
use tauri::{AppHandle, Manager};
use uplink_identity::UplinkIdentity;
use uplink_storage::{KvStore, PlatformStore};

const DB_FILE: &str = "uplink.db";
const KEY_MNEMONIC: &str = "identity_mnemonic";
const KEY_ACCOUNT: &str = "identity_account";

/// Public identity descriptor returned to the UI. Contains no secret material.
#[derive(Serialize)]
pub struct IdentityInfo {
    pub npub: String,
    pub account: u32,
}

fn open_store(app: &AppHandle, passphrase: &str) -> Result<PlatformStore, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    PlatformStore::open(&dir.join(DB_FILE), passphrase).map_err(|e| e.to_string())
}

async fn persist_identity(
    store: &PlatformStore,
    id: &UplinkIdentity,
    account: u32,
) -> Result<(), String> {
    store
        .put(KEY_MNEMONIC, id.mnemonic_phrase().as_bytes())
        .await
        .map_err(|e| e.to_string())?;
    store
        .put(KEY_ACCOUNT, &account.to_be_bytes())
        .await
        .map_err(|e| e.to_string())?;
    Ok(())
}

/// Returns the crate version — a no-secret command used to prove the native
/// command bridge is wired before any identity exists.
#[tauri::command]
pub fn app_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

/// Generate a new BIP-39 identity, persist it encrypted, and return the npub.
#[tauri::command]
pub async fn create_identity(
    app: AppHandle,
    passphrase: String,
    account: u32,
) -> Result<String, String> {
    let id = UplinkIdentity::generate(account).map_err(|e| e.to_string())?;
    let store = open_store(&app, &passphrase)?;
    persist_identity(&store, &id, account).await?;
    Ok(id.npub())
}

/// Restore an identity from a mnemonic phrase, persist it, and return the npub.
#[tauri::command]
pub async fn restore_identity(
    app: AppHandle,
    mnemonic: String,
    passphrase: String,
    account: u32,
) -> Result<String, String> {
    let id = UplinkIdentity::from_mnemonic_str(&mnemonic, account).map_err(|e| e.to_string())?;
    let store = open_store(&app, &passphrase)?;
    persist_identity(&store, &id, account).await?;
    Ok(id.npub())
}

/// Load the persisted identity (if any) and return its public descriptor.
#[tauri::command]
pub async fn current_identity(
    app: AppHandle,
    passphrase: String,
) -> Result<Option<IdentityInfo>, String> {
    let store = open_store(&app, &passphrase)?;
    let Some(mnemonic_bytes) = store.get(KEY_MNEMONIC).await.map_err(|e| e.to_string())? else {
        return Ok(None);
    };
    let mnemonic = String::from_utf8(mnemonic_bytes).map_err(|e| e.to_string())?;
    let account_bytes = store
        .get(KEY_ACCOUNT)
        .await
        .map_err(|e| e.to_string())?
        .unwrap_or_else(|| 0u32.to_be_bytes().to_vec());
    let account = u32::from_be_bytes(account_bytes.try_into().unwrap_or([0u8; 4]));
    let id = UplinkIdentity::from_mnemonic_str(&mnemonic, account).map_err(|e| e.to_string())?;
    Ok(Some(IdentityInfo {
        npub: id.npub(),
        account: id.account_index(),
    }))
}
