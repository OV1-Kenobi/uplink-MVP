import { useEffect } from "react";
import { useNavigate } from "react-router-dom";
import { useIdentityStore } from "../store/identityStore.ts";
import { useWalletStore } from "../store/walletStore.ts";

export default function DashboardPage() {
  const npub = useIdentityStore((s) => s.npub);
  const { balance, initialized, init, refresh } = useWalletStore();
  const navigate = useNavigate();
  const short = npub ? `${npub.slice(0, 14)}…${npub.slice(-8)}` : "";

  useEffect(() => {
    init();
  }, [init]);

  const sats = balance ? Math.floor(balance.lightning_msats / 1000) : null;
  const onchain = balance ? balance.onchain_confirmed_sats : null;

  return (
    <div className="page">
      <h1 className="page-title">Dashboard</h1>

      {/* Identity Card */}
      <div className="card" style={{ marginBottom: "1rem" }}>
        <div className="muted" style={{ fontSize: "0.75rem", marginBottom: "0.25rem" }}>Nostr Identity</div>
        <div className="mono" style={{ fontSize: "0.85rem", wordBreak: "break-all" }}>{short}</div>
        <button
          className="btn-ghost"
          style={{ marginTop: "0.5rem", padding: "0.3rem 0.6rem", fontSize: "0.75rem" }}
          onClick={() => { navigator.clipboard.writeText(npub ?? ""); }}
        >
          Copy npub
        </button>
      </div>

      {/* Lightning Balance */}
      <div className="card" style={{ marginBottom: "1rem" }}>
        <div className="muted" style={{ fontSize: "0.75rem", marginBottom: "0.25rem" }}>Lightning Balance</div>
        <div style={{ fontSize: "2rem", fontWeight: 700, lineHeight: 1.1 }}>
          {sats !== null ? sats.toLocaleString() : "—"}
          <span className="muted" style={{ fontSize: "1rem", marginLeft: "0.4rem" }}>sats</span>
        </div>
        {onchain !== null && onchain > 0 && (
          <div className="muted" style={{ fontSize: "0.8rem", marginTop: "0.4rem" }}>
            + {onchain.toLocaleString()} sats on-chain
          </div>
        )}
        <div style={{ display: "flex", gap: "0.5rem", marginTop: "0.75rem" }}>
          <button className="btn-primary" style={{ flex: 1, fontSize: "0.875rem" }}
            onClick={() => navigate("/wallet?tab=receive")}>
            ↓ Receive
          </button>
          <button className="btn-ghost" style={{ flex: 1, fontSize: "0.875rem" }}
            onClick={() => navigate("/wallet?tab=send")}>
            ↑ Send
          </button>
          <button className="btn-ghost" style={{ padding: "0.6rem", fontSize: "0.875rem" }}
            onClick={refresh} title="Refresh balance">
            🔄
          </button>
        </div>
      </div>

      {/* Active Streams Summary */}
      <div className="card">
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
          <span className="muted" style={{ fontSize: "0.75rem" }}>Streaming Flows</span>
          <button className="btn-ghost" style={{ padding: "0.25rem 0.6rem", fontSize: "0.75rem" }}
            onClick={() => navigate("/streams")}>
            Manage →
          </button>
        </div>
        <div className="muted" style={{ fontSize: "0.875rem" }}>
          {initialized
            ? "Open Streams tab to create or manage payment flows."
            : "Loading wallet…"}
        </div>
      </div>
    </div>
  );
}
