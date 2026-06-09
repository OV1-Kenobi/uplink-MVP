//! NIP-47 Nostr Wallet Connect adapter (ADR-U-007 §2).
//!
//! `NwcProvider` implements [`uplink_wallet::WalletProvider`] against an external
//! NIP-47 wallet. All relay I/O is behind the [`Nip47Transport`] shim so the protocol
//! layer (request encode/encrypt → response decrypt/decode) is unit-testable without a
//! live relay; the platform boundary supplies the concrete relay transport.

use async_trait::async_trait;
use nostr::nips::nip47::{
    GetInfoResponse, ListTransactionsRequest, LookupInvoiceRequest, MakeInvoiceRequest,
    NostrWalletConnectUri, PayInvoiceRequest, Request, Response, ResponseResult,
    TransactionType,
};
use nostr::Event;

use uplink_wallet::provider::{
    Invoice, InvoiceStatus, ListTxParams, PaymentResult, ProviderError, Transaction, TxKind,
    WalletBalance, WalletCapabilities, WalletInfo, WalletProvider,
};

/// Relay round-trip shim: publish an encrypted kind-23194 request, await the kind-23195
/// response event addressed to us.
#[async_trait]
pub trait Nip47Transport: Send + Sync {
    async fn request(&self, request_event: Event) -> Result<Event, crate::NostrError>;
}

/// A NIP-47 wallet exposed through the `WalletProvider` surface.
pub struct NwcProvider {
    uri: NostrWalletConnectUri,
    transport: Box<dyn Nip47Transport>,
    capabilities: WalletCapabilities,
}

impl NwcProvider {
    /// Connect using a `nostr+walletconnect://` URI string and a relay transport.
    pub fn connect(
        uri: &str,
        transport: Box<dyn Nip47Transport>,
    ) -> Result<Self, ProviderError> {
        let uri = NostrWalletConnectUri::parse(uri)
            .map_err(|e| ProviderError::Protocol(format!("invalid NWC URI: {e}")))?;
        Ok(Self::from_uri(uri, transport))
    }

    /// Build directly from a parsed URI (used by tests and pre-parsed callers).
    pub fn from_uri(uri: NostrWalletConnectUri, transport: Box<dyn Nip47Transport>) -> Self {
        let capabilities = WalletCapabilities {
            can_pay: true,
            can_make_invoice: true,
            can_lookup_invoice: true,
            can_list_transactions: true,
            supports_lnurl: uri.lud16.is_some(),
            spend_capable: true,
        };
        Self { uri, transport, capabilities }
    }

    async fn roundtrip(&self, req: Request) -> Result<ResponseResult, ProviderError> {
        let event = req
            .to_event(&self.uri)
            .map_err(|e| ProviderError::Protocol(e.to_string()))?;
        let resp_event = self
            .transport
            .request(event)
            .await
            .map_err(|e| ProviderError::Unavailable(e.to_string()))?;
        let resp = Response::from_event(&self.uri, &resp_event)
            .map_err(|e| ProviderError::Protocol(e.to_string()))?;
        if let Some(err) = resp.error {
            return Err(ProviderError::Declined(format!("{:?}: {}", err.code, err.message)));
        }
        resp.result
            .ok_or_else(|| ProviderError::Protocol("empty NIP-47 result".into()))
    }
}

fn idempotency_key(bolt11: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(bolt11.as_bytes());
    format!("nwc:{}", hex::encode(h.finalize()))
}

#[async_trait]
impl WalletProvider for NwcProvider {
    async fn get_info(&self) -> Result<WalletInfo, ProviderError> {
        let ResponseResult::GetInfo(info) = self.roundtrip(Request::get_info()).await? else {
            return Err(ProviderError::Protocol("unexpected get_info result".into()));
        };
        let info: GetInfoResponse = info;
        Ok(WalletInfo {
            node_pubkey_hex: info.pubkey.unwrap_or_default(),
            network: info.network.unwrap_or_else(|| "bitcoin".into()),
            methods: info.methods.iter().map(|m| m.as_str().to_string()).collect(),
            capabilities: self.capabilities.clone(),
        })
    }

    async fn get_balance(&self) -> Result<WalletBalance, ProviderError> {
        let ResponseResult::GetBalance(b) = self.roundtrip(Request::get_balance()).await? else {
            return Err(ProviderError::Protocol("unexpected get_balance result".into()));
        };
        Ok(WalletBalance {
            lightning_msats: b.balance,
            onchain_confirmed_sats: 0,
            stable_channel_usd_cents: None,
        })
    }

    async fn make_invoice(
        &self,
        amount_msats: u64,
        description: &str,
    ) -> Result<Invoice, ProviderError> {
        let req = Request::make_invoice(MakeInvoiceRequest {
            amount: amount_msats,
            description: Some(description.to_string()),
            description_hash: None,
            expiry: Some(3600),
        });
        let ResponseResult::MakeInvoice(r) = self.roundtrip(req).await? else {
            return Err(ProviderError::Protocol("unexpected make_invoice result".into()));
        };
        Ok(Invoice {
            bolt11: r.invoice,
            payment_hash: r.payment_hash.unwrap_or_default(),
            amount_msats: r.amount.unwrap_or(amount_msats),
            description: r.description.unwrap_or_else(|| description.to_string()),
            created_at_unix: r.created_at.map(|t| t.as_secs()).unwrap_or(0),
            expiry_seconds: 3600,
        })
    }

    async fn pay_invoice(
        &self,
        bolt11: &str,
        _max_fee_msats: Option<u64>,
    ) -> Result<PaymentResult, ProviderError> {
        if !self.capabilities.spend_capable {
            return Err(ProviderError::Declined("receive-only credential".into()));
        }
        let req = Request::pay_invoice(PayInvoiceRequest::new(bolt11));
        let ResponseResult::PayInvoice(r) = self.roundtrip(req).await? else {
            return Err(ProviderError::Protocol("unexpected pay_invoice result".into()));
        };
        let invoice_msats = parse_invoice_msats(bolt11);
        Ok(PaymentResult {
            preimage_hex: r.preimage,
            total_msats_paid: invoice_msats + r.fees_paid.unwrap_or(0),
            idempotency_key: idempotency_key(bolt11),
        })
    }

    async fn lookup_invoice(&self, payment_hash: &str) -> Result<InvoiceStatus, ProviderError> {
        let req = Request::lookup_invoice(LookupInvoiceRequest {
            payment_hash: Some(payment_hash.to_string()),
            invoice: None,
        });
        let ResponseResult::LookupInvoice(r) = self.roundtrip(req).await? else {
            return Err(ProviderError::Protocol("unexpected lookup_invoice result".into()));
        };
        Ok(InvoiceStatus {
            payment_hash: r.payment_hash,
            paid: r.settled_at.is_some(),
            preimage_hex: r.preimage,
            settled_at_unix: r.settled_at.map(|t| t.as_secs()),
        })
    }

    async fn list_transactions(
        &self,
        params: ListTxParams,
    ) -> Result<Vec<Transaction>, ProviderError> {
        let req = Request::list_transactions(ListTransactionsRequest {
            from: params.from_unix.map(nostr::Timestamp::from_secs),
            until: params.until_unix.map(nostr::Timestamp::from_secs),
            limit: params.limit.map(u64::from),
            offset: params.offset.map(u64::from),
            unpaid: Some(params.unpaid),
            transaction_type: params.kind.map(|k| match k {
                TxKind::Incoming => TransactionType::Incoming,
                TxKind::Outgoing => TransactionType::Outgoing,
            }),
        });
        let ResponseResult::ListTransactions(txs) = self.roundtrip(req).await? else {
            return Err(ProviderError::Protocol("unexpected list_transactions result".into()));
        };
        Ok(txs.into_iter().map(|t| Transaction {
            kind: match t.transaction_type {
                Some(TransactionType::Outgoing) => TxKind::Outgoing,
                _ => TxKind::Incoming,
            },
            payment_hash: t.payment_hash,
            amount_msats: t.amount,
            fees_msats: t.fees_paid,
            bolt11: t.invoice,
            preimage_hex: t.preimage,
            description: t.description,
            settled_at_unix: t.settled_at.map(|ts| ts.as_secs()),
        }).collect())
    }

    fn is_available(&self) -> bool {
        true
    }

    fn get_capabilities(&self) -> WalletCapabilities {
        self.capabilities.clone()
    }
}

/// Best-effort BOLT11 amount parse (msats); `0` if amountless or unparseable.
fn parse_invoice_msats(bolt11: &str) -> u64 {
    use std::str::FromStr;
    lightning_invoice::Bolt11Invoice::from_str(bolt11)
        .ok()
        .and_then(|i| i.amount_milli_satoshis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostr::nips::nip04;
    use nostr::nips::nip47::{
        GetBalanceResponse, MakeInvoiceResponse, Method, PayInvoiceResponse, Response,
    };
    use nostr::event::FinalizeEvent;
    use nostr::{EventBuilder, JsonUtil, Keys, Kind, RelayUrl, Tag};

    /// A mock NIP-47 wallet service: decrypts the client request and returns canned,
    /// wire-correct, NIP-04-encrypted responses signed by the wallet keypair. This
    /// exercises the full encode → encrypt → sign → decrypt → decode path.
    struct MockWallet {
        keys: Keys,
    }

    #[async_trait]
    impl Nip47Transport for MockWallet {
        async fn request(&self, req_event: Event) -> Result<Event, crate::NostrError> {
            let client_pk = req_event.pubkey;
            let json = nip04::decrypt(self.keys.secret_key(), &client_pk, req_event.content.as_str())
                .map_err(|_| crate::NostrError::Encryption)?;
            let req = Request::from_json(&json).map_err(|e| crate::NostrError::Other(e.to_string()))?;
            let result = match req.method {
                Method::GetBalance => ResponseResult::GetBalance(GetBalanceResponse { balance: 123_000 }),
                Method::MakeInvoice => ResponseResult::MakeInvoice(MakeInvoiceResponse {
                    invoice: "lnbc500n1pjmade".into(),
                    payment_hash: Some("ph_made".into()),
                    description: Some("coffee".into()),
                    description_hash: None,
                    preimage: None,
                    amount: Some(50_000),
                    created_at: None,
                    expires_at: None,
                }),
                Method::PayInvoice => ResponseResult::PayInvoice(PayInvoiceResponse {
                    preimage: "deadbeef".into(),
                    fees_paid: Some(1_000),
                }),
                other => return Err(crate::NostrError::Other(format!("unhandled {other:?}"))),
            };
            let response = Response { result_type: req.method.clone(), error: None, result: Some(result) };
            let encrypted = nip04::encrypt(self.keys.secret_key(), &client_pk, response.as_json())
                .map_err(|_| crate::NostrError::Encryption)?;
            let ev = EventBuilder::new(Kind::WalletConnectResponse, encrypted)
                .tag(Tag::public_key(client_pk))
                .tag(Tag::event(req_event.id))
                .finalize(&self.keys)
                .map_err(|e| crate::NostrError::Signing(e.to_string()))?;
            Ok(ev)
        }
    }

    fn provider() -> NwcProvider {
        let wallet = Keys::generate();
        let client = Keys::generate();
        let relay = RelayUrl::parse("wss://relay.example.com").unwrap();
        let uri = NostrWalletConnectUri::new(
            wallet.public_key(),
            vec![relay],
            client.secret_key().clone(),
            None,
        );
        NwcProvider::from_uri(uri, Box::new(MockWallet { keys: wallet }))
    }

    #[tokio::test]
    async fn nwc_get_balance_round_trips() {
        let b = provider().get_balance().await.unwrap();
        assert_eq!(b.lightning_msats, 123_000);
    }

    #[tokio::test]
    async fn nwc_make_invoice_round_trips() {
        let inv = provider().make_invoice(50_000, "coffee").await.unwrap();
        assert_eq!(inv.bolt11, "lnbc500n1pjmade");
        assert_eq!(inv.payment_hash, "ph_made");
        assert_eq!(inv.amount_msats, 50_000);
    }

    #[tokio::test]
    async fn nwc_pay_invoice_returns_preimage_through_trait() {
        let p: &dyn WalletProvider = &provider();
        let res = p.pay_invoice("lnbc1pjnoamount", Some(2_000)).await.unwrap();
        assert_eq!(res.preimage_hex, "deadbeef");
        // amountless invoice → total is just the reported fee.
        assert_eq!(res.total_msats_paid, 1_000);
        assert!(res.idempotency_key.starts_with("nwc:"));
    }
}
