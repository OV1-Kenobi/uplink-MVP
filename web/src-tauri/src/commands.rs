//! Native Tauri commands — the Tauri ⇄ UI boundary (see BOUNDARY.md, ADR-U-006).
//!
//! These wrap the same native operations `host-cli` uses. Wallet custody
//! (mnemonic, seeds, signing keys) never crosses back to the UI: identity
//! commands return only the public npub. Persistence uses the native encrypted
//! `PlatformStore` (sled) under the app data dir.

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Manager, State};
use uplink_identity::{CredentialMeta, ExternalCredential, UplinkIdentity};
use uplink_storage::{KvStore, PlatformStore};

const DB_FILE: &str = "uplink.db";
const KEY_MNEMONIC: &str = "identity_mnemonic";
const KEY_ACCOUNT: &str = "identity_account";
const KEY_CREDENTIALS: &str = "external_credentials";
const KEY_RELAYS: &str = "relay_set";

/// Public identity descriptor returned to the UI. Contains no secret material.
#[derive(Serialize)]
pub struct IdentityInfo {
    pub npub: String,
    pub account: u32,
}

/// In-memory unlocked session — the single-unlock model (ADR-U-010 §6).
///
/// Holds the device passphrase in native memory after the first unlock (create /
/// restore / `current_identity`) so post-unlock credential operations don't re-prompt
/// and the secret never re-crosses to the UI. Held as Tauri managed state and cleared
/// by `lock_session` (sign-out / app lock). The passphrase is the KEK for the at-rest
/// `PlatformStore`; it stays in the Rust layer and is never returned to the UI.
#[derive(Default)]
pub struct Session {
    passphrase: Mutex<Option<String>>,
}

impl Session {
    /// Record the passphrase for the unlocked session.
    fn set(&self, passphrase: String) {
        *self.passphrase.lock().expect("session mutex poisoned") = Some(passphrase);
    }

    /// Return the unlocked passphrase, or an error if the session is locked.
    ///
    /// Clones the value and drops the guard before returning, so callers never hold the
    /// lock across an `.await`.
    fn require(&self) -> Result<String, String> {
        self.passphrase
            .lock()
            .expect("session mutex poisoned")
            .clone()
            .ok_or_else(|| "locked: unlock the app before using connections".to_string())
    }

    /// Clear the in-memory passphrase (sign-out / app lock).
    fn clear(&self) {
        *self.passphrase.lock().expect("session mutex poisoned") = None;
    }
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
    session: State<'_, Session>,
    passphrase: String,
    account: u32,
) -> Result<String, String> {
    let id = UplinkIdentity::generate(account).map_err(|e| e.to_string())?;
    let store = open_store(&app, &passphrase)?;
    persist_identity(&store, &id, account).await?;
    session.set(passphrase);
    Ok(id.npub())
}

/// Restore an identity from a mnemonic phrase, persist it, and return the npub.
#[tauri::command]
pub async fn restore_identity(
    app: AppHandle,
    session: State<'_, Session>,
    mnemonic: String,
    passphrase: String,
    account: u32,
) -> Result<String, String> {
    let id = UplinkIdentity::from_mnemonic_str(&mnemonic, account).map_err(|e| e.to_string())?;
    let store = open_store(&app, &passphrase)?;
    persist_identity(&store, &id, account).await?;
    session.set(passphrase);
    Ok(id.npub())
}

/// Load the persisted identity (if any) and return its public descriptor.
#[tauri::command]
pub async fn current_identity(
    app: AppHandle,
    session: State<'_, Session>,
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
    // Decryption succeeded, so the passphrase is correct: hold it for the session.
    session.set(passphrase);
    Ok(Some(IdentityInfo {
        npub: id.npub(),
        account: id.account_index(),
    }))
}

/// Passphrase-free probe: is an identity already provisioned on this device?
///
/// Checks key existence in the native store without decrypting, so the
/// onboarding flow can route to the unlock screen at launch (the native
/// counterpart to the wasm `localStorage` existence check).
#[tauri::command]
pub async fn has_identity(app: AppHandle) -> Result<bool, String> {
    let store = open_store(&app, "")?;
    store.exists(KEY_MNEMONIC).await.map_err(|e| e.to_string())
}

/// Clear the provisioned identity from the native store ("Reset app").
///
/// Removes the persisted identity keys so the device returns to the
/// unprovisioned state. Returns no secret material and needs no passphrase —
/// deletion removes the stored ciphertext without decrypting it (the native
/// counterpart to the wasm `localStorage.clear()` reset path).
#[tauri::command]
pub async fn reset_identity(app: AppHandle) -> Result<(), String> {
    let store = open_store(&app, "")?;
    store.delete(KEY_MNEMONIC).await.map_err(|e| e.to_string())?;
    store.delete(KEY_ACCOUNT).await.map_err(|e| e.to_string())?;
    Ok(())
}

/// Decrypt and return the mnemonic word list for the one-time backup screen.
///
/// This is the native counterpart to the wasm `export_mnemonic_words` boundary
/// export — the single sanctioned path by which the mnemonic reaches the UI. Uses the
/// unlocked session passphrase (see BOUNDARY.md, ADR-U-006).
#[tauri::command]
pub async fn export_mnemonic(
    app: AppHandle,
    session: State<'_, Session>,
) -> Result<Vec<String>, String> {
    let passphrase = session.require()?;
    let store = open_store(&app, &passphrase)?;
    let mnemonic_bytes = store
        .get(KEY_MNEMONIC)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "no identity provisioned".to_string())?;
    let mnemonic = String::from_utf8(mnemonic_bytes).map_err(|e| e.to_string())?;
    Ok(mnemonic.split_whitespace().map(|w| w.to_string()).collect())
}

// ─────────────────────────────────────────────────────────────────────────────
// Phase 5a — external credentials + relay set (ADR-U-010)
//
// External credentials (NWC URIs, LNC pairing phrases, Lightning addresses, npub /
// NIP-05) are stored in the same passphrase-encrypted `PlatformStore` as the mnemonic.
// Bearer secrets never cross back to the UI: every command returns only the redacted,
// non-secret `CredentialMeta` (BOUNDARY.md, ADR-U-006, ADR-U-010 custody invariant).
// ─────────────────────────────────────────────────────────────────────────────

/// A credential as persisted at rest: the full (possibly secret) credential plus the
/// time it was linked. Never serialized to the UI — only its `meta()` is.
#[derive(Serialize, Deserialize)]
struct StoredCredential {
    credential: ExternalCredential,
    added_at_unix: u64,
}

fn now_unix() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Stable snake_case string for a credential kind (matches the serde representation),
/// used as the disconnect selector from the UI.
fn kind_str(kind: uplink_identity::CredentialKind) -> String {
    serde_json::to_value(kind)
        .ok()
        .and_then(|v| v.as_str().map(str::to_string))
        .unwrap_or_default()
}

async fn load_credentials(store: &PlatformStore) -> Result<Vec<StoredCredential>, String> {
    match store.get(KEY_CREDENTIALS).await.map_err(|e| e.to_string())? {
        Some(bytes) => serde_json::from_slice(&bytes).map_err(|e| e.to_string()),
        None => Ok(Vec::new()),
    }
}

async fn save_credentials(store: &PlatformStore, creds: &[StoredCredential]) -> Result<(), String> {
    let bytes = serde_json::to_vec(creds).map_err(|e| e.to_string())?;
    store.put(KEY_CREDENTIALS, &bytes).await.map_err(|e| e.to_string())
}

/// Insert (or replace the same-kind entry) and return the redacted descriptor.
async fn upsert_credential(
    app: &AppHandle,
    passphrase: &str,
    credential: ExternalCredential,
) -> Result<CredentialMeta, String> {
    let store = open_store(app, passphrase)?;
    let mut creds = load_credentials(&store).await?;
    let kind = credential.kind();
    creds.retain(|c| c.credential.kind() != kind);
    let added_at_unix = now_unix();
    let meta = credential.meta(added_at_unix);
    creds.push(StoredCredential { credential, added_at_unix });
    save_credentials(&store, &creds).await?;
    Ok(meta)
}

/// Link an NWC connection string (receive + spend — ADR-U-010 §2).
#[tauri::command]
pub async fn connect_nwc(
    app: AppHandle,
    session: State<'_, Session>,
    uri: String,
) -> Result<CredentialMeta, String> {
    let passphrase = session.require()?;
    let cred = ExternalCredential::nwc(&uri).map_err(|e| e.to_string())?;
    upsert_credential(&app, &passphrase, cred).await
}

/// Link a Lightning Node Connect 10-word pairing phrase (spend; LND-direct, gated).
#[tauri::command]
pub async fn connect_lnc(
    app: AppHandle,
    session: State<'_, Session>,
    pairing_phrase: String,
) -> Result<CredentialMeta, String> {
    let passphrase = session.require()?;
    let cred = ExternalCredential::lnc(&pairing_phrase).map_err(|e| e.to_string())?;
    upsert_credential(&app, &passphrase, cred).await
}

/// Set the user's own Lightning Address (primary receive path).
#[tauri::command]
pub async fn set_lightning_address(
    app: AppHandle,
    session: State<'_, Session>,
    address: String,
) -> Result<CredentialMeta, String> {
    let passphrase = session.require()?;
    let cred = ExternalCredential::lightning_address(&address).map_err(|e| e.to_string())?;
    upsert_credential(&app, &passphrase, cred).await
}

/// Link an existing Nostr identity (`kind` = "npub" or "nip05").
#[tauri::command]
pub async fn link_identity(
    app: AppHandle,
    session: State<'_, Session>,
    kind: String,
    value: String,
) -> Result<CredentialMeta, String> {
    let passphrase = session.require()?;
    let cred = match kind.as_str() {
        "npub" => ExternalCredential::npub(&value),
        "nip05" => ExternalCredential::nip05(&value),
        other => return Err(format!("unsupported identity kind: {other}")),
    }
    .map_err(|e| e.to_string())?;
    upsert_credential(&app, &passphrase, cred).await
}

/// List linked credentials as redacted, non-secret descriptors.
#[tauri::command]
pub async fn list_credentials(
    app: AppHandle,
    session: State<'_, Session>,
) -> Result<Vec<CredentialMeta>, String> {
    let passphrase = session.require()?;
    let store = open_store(&app, &passphrase)?;
    let creds = load_credentials(&store).await?;
    Ok(creds.iter().map(|c| c.credential.meta(c.added_at_unix)).collect())
}

/// Remove a linked credential by its kind string (e.g. "nwc", "lnc").
#[tauri::command]
pub async fn disconnect_credential(
    app: AppHandle,
    session: State<'_, Session>,
    kind: String,
) -> Result<(), String> {
    let passphrase = session.require()?;
    let store = open_store(&app, &passphrase)?;
    let mut creds = load_credentials(&store).await?;
    creds.retain(|c| kind_str(c.credential.kind()) != kind);
    save_credentials(&store, &creds).await
}

/// Return the persisted relay set, or `None` if the user has never customized it
/// (the UI then falls back to the built-in default set — ADR-U-010 §5).
///
/// Relay URLs are non-secret, so this is passphrase-less: Settings can manage relays
/// without re-prompting for the device password (which is not retained after unlock).
#[tauri::command]
pub async fn get_relays(app: AppHandle) -> Result<Option<Vec<String>>, String> {
    let store = open_store(&app, "")?;
    match store.get(KEY_RELAYS).await.map_err(|e| e.to_string())? {
        Some(bytes) => Ok(Some(serde_json::from_slice(&bytes).map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

/// Persist the user's relay set (passphrase-less; relay URLs are non-secret).
#[tauri::command]
pub async fn set_relays(app: AppHandle, relays: Vec<String>) -> Result<(), String> {
    let store = open_store(&app, "")?;
    let bytes = serde_json::to_vec(&relays).map_err(|e| e.to_string())?;
    store.put(KEY_RELAYS, &bytes).await.map_err(|e| e.to_string())
}

/// Lock the session: clear the in-memory passphrase (sign-out / app lock).
///
/// After this, credential operations require a fresh unlock. Returns no secret material.
#[tauri::command]
pub fn lock_session(session: State<'_, Session>) {
    session.clear();
}
