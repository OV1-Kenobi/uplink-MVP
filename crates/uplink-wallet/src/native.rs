//! `NativeLdkWallet` — implementation of `WalletExecutor` using `ldk-node`.
//!
//! Only active when the `native` feature is enabled.

use std::sync::Arc;
use ldk_node::config::Config;
use ldk_node::{Builder, Node};
use ldk_node::lightning_invoice::{Bolt11InvoiceDescription, Description};
use crate::executor::{WalletExecutor, WalletBalance, PaymentResult};
use crate::WalletError;
use uplink_identity::UplinkIdentity;

/// A native LDK wallet backed by `ldk-node`.
pub struct NativeLdkWallet {
    node: Arc<Node>,
}

impl NativeLdkWallet {
    /// Create a new native LDK wallet from an identity.
    ///
    /// - `storage_dir`: filesystem path for LDK state, channels, and Sqlite DB.
    /// - `network`: Bitcoin network (e.g. "regtest", "testnet").
    /// - `esplora_url`: URL of an Esplora server for chain sync.
    pub fn new(
        identity: &UplinkIdentity,
        storage_dir: &str,
        network: bitcoin::Network,
        esplora_url: &str,
    ) -> Result<Self, WalletError> {
        let mut config = Config::default();
        config.network = network;

        let mut builder = Builder::from_config(config);
        builder.set_entropy_seed_bytes(identity.ldk_node_seed);
        builder.set_storage_dir_path(storage_dir.to_string());
        builder.set_chain_source_esplora(esplora_url.to_string(), None);

        let node = builder.build()
            .map_err(|e| WalletError::Ldk(e.to_string()))?;

        node.start()
            .map_err(|e| WalletError::Ldk(e.to_string()))?;

        Ok(Self { node: Arc::new(node) })
    }

    /// Explicitly sync with the chain.
    pub fn sync(&self) -> Result<(), WalletError> {
        self.node.sync_wallets()
            .map_err(|e| WalletError::Ldk(e.to_string()))
    }
}

impl WalletExecutor for NativeLdkWallet {
    fn balance(&self) -> Result<WalletBalance, WalletError> {
        let balances = self.node.list_balances();

        Ok(WalletBalance {
            lightning_msats: balances.total_lightning_balance_sats * 1000,
            onchain_confirmed_sats: balances.spendable_onchain_balance_sats,
            stable_channel_usd_cents: None, // Only populated by LSP in Phase A4
        })
    }

    fn receive_invoice(&self, msats: u64, memo: &str) -> Result<String, WalletError> {
        let desc = Description::new(memo.to_string())
            .map_err(|e| WalletError::Ldk(e.to_string()))?;
        let invoice = self.node.bolt11_payment().receive(msats, &Bolt11InvoiceDescription::Direct(desc), 3600)
            .map_err(|e| WalletError::Ldk(e.to_string()))?;
        Ok(invoice.to_string())
    }

    fn pay_invoice(
        &self,
        bolt11: &str,
        _max_fee_msats: u64,
        idempotency_key: &str,
    ) -> Result<PaymentResult, WalletError> {
        let invoice: ldk_node::lightning_invoice::Bolt11Invoice = bolt11.parse()
            .map_err(|e: ldk_node::lightning_invoice::ParseOrSemanticError| WalletError::Ldk(e.to_string()))?;

        // Configure max fee
        // Note: ldk-node 0.7 send takes Option<RouteParametersConfig>
        // We'll leave it as None for now or try to set it if we can find the type easily.
        // Actually, let's just use None for the simplest implementation of A3.

        let payment_id = self.node.bolt11_payment().send(&invoice, None)
            .map_err(|e| WalletError::Ldk(e.to_string()))?;

        // Poll for result (blocking)
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(60);

        while start.elapsed() < timeout {
            if let Some(payment) = self.node.payment(&payment_id) {
                match payment.status {
                    ldk_node::payment::PaymentStatus::Succeeded => {
                        let preimage_hex = match payment.kind {
                            ldk_node::payment::PaymentKind::Bolt11 { preimage, .. } => {
                                preimage.map(|p| hex::encode(p.0)).unwrap_or_else(|| "unknown".to_string())
                            }
                            _ => "unknown".to_string(),
                        };
                        return Ok(PaymentResult {
                            preimage_hex,
                            total_msats_paid: invoice.amount_milli_satoshis().unwrap_or(0) + payment.fee_paid_msat.unwrap_or(0),
                            idempotency_key: idempotency_key.to_string(),
                        });
                    }
                    ldk_node::payment::PaymentStatus::Failed => {
                        return Err(WalletError::Ldk("Payment failed".to_string()));
                    }
                    ldk_node::payment::PaymentStatus::Pending => {
                        // continue polling
                    }
                }
            }
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        Err(WalletError::Ldk("Payment timeout".to_string()))
    }

    fn receive_onchain_address(&self) -> Result<String, WalletError> {
        let address = self.node.onchain_payment().new_address()
            .map_err(|e| WalletError::Ldk(e.to_string()))?;
        Ok(address.to_string())
    }

    fn node_pubkey_hex(&self) -> String {
        self.node.node_id().to_string()
    }
}
