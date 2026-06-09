//! Lightning Node Connect (LNC) spend provider (ADR-U-010 §4).
//!
//! LNC is the spend path for LND nodes. There is no mature pure-Rust LNC client today —
//! the reference implementation is Go compiled to WASM (mailbox proxy + PAKE/brontide
//! `Noise_XK_secp256k1_ChaChaPoly_SHA256` + gRPC tunneled over WebSocket). So in Phase 5a
//! the transport is **gated**: the 10-word pairing phrase is captured and held for the
//! future transport, but every network method returns [`ProviderError::Unavailable`]
//! rather than a `todo!()` stub (mirrors the LSP gating, ADR-U-002). Implementing the
//! transport is the first task of Phase 9 / 5b.
//!
//! ## Custody (ADR-U-006, ADR-U-010)
//! The pairing phrase is a bearer secret. It is held only inside Rust, never logged, and
//! never returned to the UI. `Debug` is redacted.

use async_trait::async_trait;

use crate::provider::{
    Invoice, InvoiceStatus, ListTxParams, PaymentResult, ProviderError, Transaction,
    WalletBalance, WalletCapabilities, WalletInfo, WalletProvider,
};

/// Message used for every gated network method until the LNC transport lands.
const TRANSPORT_GATED: &str = "LNC transport not yet wired (ADR-U-010 §4)";

/// A Lightning Node Connect spend provider for an LND node.
///
/// Holds the pairing phrase for the (future) transport; advertises spend capability so a
/// capability-aware UI offers spending, while all network calls are gated until the
/// transport is implemented.
pub struct LncProvider {
    pairing_phrase: String,
}

impl LncProvider {
    /// Construct from a 10-word LNC pairing phrase. Validation of word-count is performed
    /// upstream by `uplink_identity::ExternalCredential::lnc`.
    pub fn new(pairing_phrase: impl Into<String>) -> Self {
        Self { pairing_phrase: pairing_phrase.into() }
    }

    /// Non-secret diagnostic: the number of words in the held pairing phrase.
    /// Does not reveal the phrase itself.
    #[must_use]
    pub fn pairing_word_count(&self) -> usize {
        self.pairing_phrase.split_whitespace().count()
    }
}

/// Redacting `Debug`: the pairing phrase is never printed.
impl std::fmt::Debug for LncProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LncProvider")
            .field("pairing_phrase", &"<redacted>")
            .finish()
    }
}

fn gated<T>() -> Result<T, ProviderError> {
    Err(ProviderError::Unavailable(TRANSPORT_GATED.into()))
}

#[async_trait]
impl WalletProvider for LncProvider {
    async fn get_info(&self) -> Result<WalletInfo, ProviderError> {
        gated()
    }
    async fn get_balance(&self) -> Result<WalletBalance, ProviderError> {
        gated()
    }
    async fn make_invoice(&self, _amount_msats: u64, _description: &str)
        -> Result<Invoice, ProviderError> {
        gated()
    }
    async fn pay_invoice(&self, _bolt11: &str, _max_fee_msats: Option<u64>)
        -> Result<PaymentResult, ProviderError> {
        gated()
    }
    async fn lookup_invoice(&self, _payment_hash: &str)
        -> Result<InvoiceStatus, ProviderError> {
        gated()
    }
    async fn list_transactions(&self, _params: ListTxParams)
        -> Result<Vec<Transaction>, ProviderError> {
        gated()
    }

    fn is_available(&self) -> bool {
        // Transport not yet wired; the provider is configured but not reachable.
        false
    }

    fn get_capabilities(&self) -> WalletCapabilities {
        // LNC is the spend rail: advertise spend so a capability-aware UI offers it, even
        // though the transport is gated. Spending attempts surface `Unavailable`, not a
        // silent failure.
        WalletCapabilities {
            can_pay: true,
            can_make_invoice: true,
            can_lookup_invoice: false,
            can_list_transactions: false,
            supports_lnurl: false,
            spend_capable: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PHRASE: &str = "one two three four five six seven eight nine ten";

    #[test]
    fn advertises_spend_but_is_not_yet_available() {
        let p = LncProvider::new(PHRASE);
        let caps = p.get_capabilities();
        assert!(caps.spend_capable && caps.can_pay);
        assert!(!p.is_available());
        assert_eq!(p.pairing_word_count(), 10);
    }

    #[tokio::test]
    async fn spend_is_gated_not_stubbed() {
        let p = LncProvider::new(PHRASE);
        match p.pay_invoice("lnbc1pjexample", None).await {
            Err(ProviderError::Unavailable(msg)) => assert!(msg.contains("LNC transport")),
            other => panic!("expected Unavailable, got {other:?}"),
        }
        assert!(matches!(p.get_balance().await, Err(ProviderError::Unavailable(_))));
    }

    #[test]
    fn debug_redacts_pairing_phrase() {
        let dbg = format!("{:?}", LncProvider::new(PHRASE));
        assert!(dbg.contains("<redacted>"));
        assert!(!dbg.contains("seven"));
    }
}
