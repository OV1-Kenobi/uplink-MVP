/**
 * WalletPage — Receive (on-chain address + BOLT11 invoice) and Send (pay invoice).
 * Only calls wasm via uplink-client.ts — never calls fetch/WebSocket directly.
 */
import { useState, useEffect } from "react";
import { useSearchParams } from "react-router-dom";
import { getReceiveAddress, getInvoice, payInvoice } from "../wasm/uplink-client.ts";
import { useWalletStore } from "../store/walletStore.ts";

type Tab = "receive" | "send";

export default function WalletPage() {
  const [params] = useSearchParams();
  const [tab, setTab] = useState<Tab>((params.get("tab") as Tab) ?? "receive");
  const refresh = useWalletStore((s) => s.refresh);

  // Receive state
  const [address, setAddress] = useState<string | null>(null);
  const [invoice, setInvoice] = useState<string | null>(null);
  const [invoiceSats, setInvoiceSats] = useState("1000");
  const [invoiceMemo, setInvoiceMemo] = useState("Uplink top-up");
  const [receiveError, setReceiveError] = useState<string | null>(null);

  // Send state
  const [bolt11Input, setBolt11Input] = useState("");
  const [sendResult, setSendResult] = useState<string | null>(null);
  const [sendError, setSendError] = useState<string | null>(null);
  const [sending, setSending] = useState(false);

  useEffect(() => { getReceiveAddress().then(setAddress).catch(console.warn); }, []);

  async function handleGetInvoice(e: React.FormEvent) {
    e.preventDefault();
    setReceiveError(null);
    setInvoice(null);
    try {
      const msats = Math.round(parseFloat(invoiceSats) * 1000);
      const inv = await getInvoice(msats, invoiceMemo);
      setInvoice(inv);
    } catch (err) {
      setReceiveError(String(err));
    }
  }

  async function handleSend(e: React.FormEvent) {
    e.preventDefault();
    setSendError(null);
    setSendResult(null);
    setSending(true);
    try {
      const key = `cli-${Date.now()}`;
      const result = await payInvoice(bolt11Input.trim(), 5000, key);
      setSendResult(`✓ Paid! Preimage: ${result.preimage_hex.slice(0, 16)}… (${result.total_msats_paid} msats)`);
      await refresh();
    } catch (err) {
      setSendError(String(err));
    } finally {
      setSending(false);
    }
  }

  return (
    <div className="page">
      <h1 className="page-title">Wallet</h1>

      <div style={{ display: "flex", gap: "0.5rem", marginBottom: "1.25rem" }}>
        {(["receive", "send"] as Tab[]).map((t) => (
          <button key={t} className={tab === t ? "btn-primary" : "btn-ghost"}
            style={{ flex: 1, textTransform: "capitalize" }} onClick={() => setTab(t)}>
            {t === "receive" ? "↓ Receive" : "↑ Send"}
          </button>
        ))}
      </div>

      {tab === "receive" && (
        <>
          <div className="card" style={{ marginBottom: "1rem" }}>
            <div className="muted" style={{ fontSize: "0.75rem", marginBottom: "0.5rem" }}>On-chain Address</div>
            {address
              ? <div className="mono" style={{ fontSize: "0.8rem", wordBreak: "break-all" }}>{address}</div>
              : <div className="muted">Loading address…</div>}
            {address && (
              <button className="btn-ghost" style={{ marginTop: "0.5rem", fontSize: "0.75rem", padding: "0.3rem 0.6rem" }}
                onClick={() => navigator.clipboard.writeText(address)}>Copy</button>
            )}
          </div>
          <div className="card">
            <div className="muted" style={{ fontSize: "0.75rem", marginBottom: "0.75rem" }}>Lightning Invoice (BOLT11)</div>
            <form onSubmit={handleGetInvoice}>
              <input placeholder="Amount (sats)" type="number" min="1" value={invoiceSats}
                onChange={(e) => setInvoiceSats(e.target.value)} style={{ marginBottom: "0.5rem" }} />
              <input placeholder="Memo" value={invoiceMemo}
                onChange={(e) => setInvoiceMemo(e.target.value)} style={{ marginBottom: "0.75rem" }} />
              <button type="submit" className="btn-primary">Generate Invoice</button>
            </form>
            {receiveError && <div style={{ color: "var(--danger)", marginTop: "0.75rem", fontSize: "0.85rem" }}>{receiveError}</div>}
            {invoice && (
              <div style={{ marginTop: "0.75rem" }}>
                <div className="mono" style={{ fontSize: "0.65rem", wordBreak: "break-all", background: "var(--surface-2)", padding: "0.75rem", borderRadius: "8px" }}>
                  {invoice}
                </div>
                <button className="btn-ghost" style={{ marginTop: "0.5rem", fontSize: "0.75rem", padding: "0.3rem 0.6rem" }}
                  onClick={() => navigator.clipboard.writeText(invoice)}>Copy Invoice</button>
              </div>
            )}
          </div>
        </>
      )}

      {tab === "send" && (
        <div className="card">
          <div className="muted" style={{ fontSize: "0.75rem", marginBottom: "0.75rem" }}>Pay BOLT11 Invoice</div>
          <form onSubmit={handleSend}>
            <textarea placeholder="lnbc…" value={bolt11Input} onChange={(e) => setBolt11Input(e.target.value)}
              rows={4} style={{ width: "100%", fontFamily: "monospace", fontSize: "0.75rem", resize: "vertical", marginBottom: "0.75rem" }} />
            <button type="submit" className="btn-primary" disabled={sending || !bolt11Input.trim()}>
              {sending ? "Sending…" : "Pay Invoice"}
            </button>
          </form>
          {sendError && <div style={{ color: "var(--danger)", marginTop: "0.75rem", fontSize: "0.85rem" }}>{sendError}</div>}
          {sendResult && <div style={{ color: "var(--success)", marginTop: "0.75rem", fontSize: "0.85rem" }}>{sendResult}</div>}
        </div>
      )}
    </div>
  );
}
