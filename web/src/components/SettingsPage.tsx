import { useState } from "react";
import { useIdentityStore } from "../store/identityStore.ts";

const DEFAULT_RELAYS = [
  "wss://relay.damus.io",
  "wss://nos.lol",
  "wss://relay.nostr.band",
  "wss://relay.openagents.com",
];

export default function SettingsPage() {
  const { npub, clearIdentity } = useIdentityStore();
  const [relays, setRelays] = useState<string[]>(DEFAULT_RELAYS);
  const [newRelay, setNewRelay] = useState("");

  function addRelay(e: React.FormEvent) {
    e.preventDefault();
    if (newRelay.trim() && !relays.includes(newRelay.trim())) {
      setRelays((r) => [...r, newRelay.trim()]);
      setNewRelay("");
    }
  }

  return (
    <div className="page">
      <h1 className="page-title">Settings</h1>

      <section className="card" style={{ marginBottom: "1.25rem" }}>
        <h3 style={{ margin: "0 0 0.75rem" }}>Identity</h3>
        <div className="muted" style={{ marginBottom: "0.25rem" }}>Your npub</div>
        <div className="mono" style={{ wordBreak: "break-all", fontSize: "0.75rem", marginBottom: "1rem" }}>{npub}</div>
        <button
          className="btn-ghost"
          style={{ color: "var(--danger)", borderColor: "var(--danger)" }}
          onClick={() => { if (confirm("Sign out? Your mnemonic backup is required to restore.")) clearIdentity(); }}
        >
          Sign out
        </button>
      </section>

      <section className="card" style={{ marginBottom: "1.25rem" }}>
        <h3 style={{ margin: "0 0 0.75rem" }}>Nostr Relays</h3>
        <div className="muted" style={{ marginBottom: "0.75rem", fontSize: "0.8rem" }}>
          Receipt events (kind 9901) are published to your primary relay.
        </div>
        {relays.map((r) => (
          <div key={r} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.5rem" }}>
            <span className="mono" style={{ fontSize: "0.8rem" }}>{r}</span>
            {!DEFAULT_RELAYS.includes(r) && (
              <button className="btn-ghost" style={{ padding: "0.2rem 0.5rem", fontSize: "0.75rem" }}
                onClick={() => setRelays((rs) => rs.filter((x) => x !== r))}>
                Remove
              </button>
            )}
          </div>
        ))}
        <form onSubmit={addRelay} style={{ display: "flex", gap: "0.5rem", marginTop: "0.75rem" }}>
          <input placeholder="wss://your-relay.com" value={newRelay}
            onChange={(e) => setNewRelay(e.target.value)} style={{ flex: 1, fontSize: "0.875rem" }} />
          <button type="submit" className="btn-primary" style={{ width: "auto", padding: "0.6rem 1rem" }}>Add</button>
        </form>
      </section>

      <section className="card">
        <h3 style={{ margin: "0 0 0.5rem" }}>LSP</h3>
        <div className="muted" style={{ fontSize: "0.875rem" }}>
          Stable-Channels LSP integration available in Phase A4.
          LSP node pubkey and connection details will be configurable here.
        </div>
      </section>
    </div>
  );
}
