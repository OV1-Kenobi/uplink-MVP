//! `ExecutorProvider` — adapts any synchronous [`WalletExecutor`] (today the embedded
//! `NativeLdkWallet`) to the async [`WalletProvider`] surface (ADR-U-007 §2).
//!
//! Methods the executor cannot serve (`lookup_invoice`, `list_transactions`) return
//! [`ProviderError::Unsupported`]. The `spend_capable` flag scaffolds the two-credential
//! split (ADR-U-007 §4).

use async_trait::async_trait;
use lightning_invoice::Bolt11Invoice;
use std::str::FromStr;

use crate::executor::{PaymentResult, WalletBalance, WalletExecutor};
use crate::provider::{
    Invoice, InvoiceStatus, ListTxParams, ProviderError, Transaction, WalletCapabilities,
    WalletInfo, WalletProvider,
};

/// Wraps a `WalletExecutor` (e.g. the embedded LDK node) as a `WalletProvider`.
pub struct ExecutorProvider<W: WalletExecutor> {
    inner: W,
    network: String,
    spend_capable: bool,
}

impl<W: WalletExecutor> ExecutorProvider<W> {
    /// Wrap `executor` as a spend-capable provider on `network` (e.g. "bitcoin").
    pub fn new(executor: W, network: impl Into<String>) -> Self {
        Self { inner: executor, network: network.into(), spend_capable: true }
    }

    /// Builder: mark this provider receive-only (cannot pay).
    pub fn receive_only(mut self) -> Self {
        self.spend_capable = false;
        self
    }

    fn capabilities(&self) -> WalletCapabilities {
        WalletCapabilities {
            can_pay: self.spend_capable,
            can_make_invoice: true,
            can_lookup_invoice: false,
            can_list_transactions: false,
            supports_lnurl: false,
            spend_capable: self.spend_capable,
        }
    }
}

#[async_trait]
impl<W: WalletExecutor> WalletProvider for ExecutorProvider<W> {
    async fn get_info(&self) -> Result<WalletInfo, ProviderError> {
        Ok(WalletInfo {
            node_pubkey_hex: self.inner.node_pubkey_hex(),
            network: self.network.clone(),
            methods: vec![
                "get_info".into(),
                "get_balance".into(),
                "make_invoice".into(),
                "pay_invoice".into(),
            ],
            capabilities: self.capabilities(),
        })
    }

    async fn get_balance(&self) -> Result<WalletBalance, ProviderError> {
        Ok(self.inner.balance()?)
    }

    async fn make_invoice(
        &self,
        amount_msats: u64,
        description: &str,
    ) -> Result<Invoice, ProviderError> {
        let bolt11 = self.inner.receive_invoice(amount_msats, description)?;
        // Best-effort parse for the payment hash; the bolt11 string is authoritative.
        let (payment_hash, parsed_msats) = match Bolt11Invoice::from_str(&bolt11) {
            Ok(inv) => (
                hex::encode(inv.payment_hash().as_ref() as &[u8]),
                inv.amount_milli_satoshis().unwrap_or(amount_msats),
            ),
            Err(_) => (String::new(), amount_msats),
        };
        Ok(Invoice {
            bolt11,
            payment_hash,
            amount_msats: parsed_msats,
            description: description.to_string(),
            created_at_unix: 0,
            expiry_seconds: 3600,
        })
    }

    async fn pay_invoice(
        &self,
        bolt11: &str,
        max_fee_msats: Option<u64>,
    ) -> Result<PaymentResult, ProviderError> {
        if !self.spend_capable {
            return Err(ProviderError::Declined("receive-only credential".into()));
        }
        let key = format!("exec:{bolt11}");
        let res = self
            .inner
            .pay_invoice(bolt11, max_fee_msats.unwrap_or(u64::MAX), &key)?;
        Ok(res)
    }

    async fn lookup_invoice(&self, _payment_hash: &str) -> Result<InvoiceStatus, ProviderError> {
        Err(ProviderError::Unsupported)
    }

    async fn list_transactions(
        &self,
        _params: ListTxParams,
    ) -> Result<Vec<Transaction>, ProviderError> {
        Err(ProviderError::Unsupported)
    }

    fn is_available(&self) -> bool {
        true
    }

    fn get_capabilities(&self) -> WalletCapabilities {
        self.capabilities()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::StubWallet;

    #[tokio::test]
    async fn stub_executor_exposes_provider_surface() {
        let p = ExecutorProvider::new(StubWallet, "regtest");
        let info = p.get_info().await.unwrap();
        assert_eq!(info.network, "regtest");
        assert!(info.capabilities.spend_capable);
        assert!(p.is_available());
        // Unsupported executor methods surface as Unsupported, not panics.
        assert!(matches!(
            p.lookup_invoice("ph").await,
            Err(ProviderError::Unsupported)
        ));
    }

    #[tokio::test]
    async fn receive_only_declines_payment() {
        let p = ExecutorProvider::new(StubWallet, "regtest").receive_only();
        assert!(!p.get_capabilities().spend_capable);
        assert!(matches!(
            p.pay_invoice("lnbc1...", None).await,
            Err(ProviderError::Declined(_))
        ));
    }
}
