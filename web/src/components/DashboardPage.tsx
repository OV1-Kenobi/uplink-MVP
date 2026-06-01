import { useIdentityStore } from "../store/identityStore.ts";

export default function DashboardPage() {
  const npub = useIdentityStore((s) => s.npub);
  const short = npub ? `${npub.slice(0, 12)}…${npub.slice(-8)}` : "";

  return (
    <div className="page">
      <h1 className="page-title">Dashboard</h1>

      <div className="card" style={{ marginBottom: "1rem" }}>
        <div className="muted" style={{ marginBottom: "0.25rem" }}>Your Nostr identity</div>
        <div className="mono">{short}</div>
      </div>

      <div className="card" style={{ marginBottom: "1rem" }}>
        <div className="muted" style={{ marginBottom: "0.25rem" }}>Lightning balance</div>
        <div style={{ fontSize: "1.8rem", fontWeight: 700 }}>
          — <span className="muted" style={{ fontSize: "1rem" }}>sats</span>
        </div>
        <div className="muted" style={{ fontSize: "0.8rem", marginTop: "0.25rem" }}>
          Stable-Channel wallet available in Phase A3
        </div>
      </div>

      <div className="card">
        <div className="muted" style={{ marginBottom: "0.5rem" }}>Active streams</div>
        <div className="muted" style={{ fontSize: "0.875rem" }}>
          No active streams. Go to <strong>Streams</strong> to create one.
        </div>
      </div>
    </div>
  );
}
