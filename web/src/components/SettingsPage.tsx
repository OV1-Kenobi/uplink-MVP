import { useState, useEffect } from "react";
import { useIdentityStore } from "../store/identityStore.ts";
import { createDelegation, DelegationToken } from "../wasm/uplink-client.ts";
import { getRelays, setRelays as persistRelaySet, listCredentials, disconnectCredential, lockSession } from "../identity.ts";
import type { CredentialMeta } from "../identity.ts";


const DEFAULT_RELAYS = [
  "wss://relay.damus.io",
  "wss://relay.primal.net",
  "wss://nos.lol",
];

export default function SettingsPage() {
  const { npub, clearIdentity } = useIdentityStore();

  const [delegations, setDelegations] = useState<DelegationToken[]>([]);
  const [showDelegationForm, setShowDelegationForm] = useState(false);
  const [delChildNpub, setDelChildNpub] = useState("");
  const [delAmount, setDelAmount] = useState("10000");

  async function handleCreateDelegation(e: React.FormEvent) {
    e.preventDefault();
    try {
      const token = await createDelegation({
        childNpub: delChildNpub,
        childWalletId: "mobile-agent-1",
        maxPerTxSats: parseInt(delAmount),
        rolling24hCapSats: parseInt(delAmount) * 5,
        expiresAtUnix: Math.floor(Date.now() / 1000) + 86400 * 30, // 30 days
      });
      setDelegations([...delegations, token]);
      setShowDelegationForm(false);
      setDelChildNpub("");
      alert(`Delegation token created for ${delChildNpub.slice(0, 12)}...`);
    } catch (err: any) {
      alert(`Error creating delegation: ${err.message || err}`);
    }
  }

  const [relays, setRelays] = useState<string[]>(DEFAULT_RELAYS);
  const [newRelay, setNewRelay] = useState("");

  // Load the persisted relay set on mount; fall back to the built-in defaults.
  useEffect(() => {
    getRelays()
      .then((stored) => { if (stored && stored.length) setRelays(stored); })
      .catch(() => {});
  }, []);

  function saveRelays(next: string[]) {
    setRelays(next);
    persistRelaySet(next).catch(() => {});
  }

  function addRelay(e: React.FormEvent) {
    e.preventDefault();
    const r = newRelay.trim();
    if (r && !relays.includes(r)) {
      saveRelays([...relays, r]);
      setNewRelay("");
    }
  }

  // Connections (external credentials) — loaded via the unlocked session (single-unlock
  // model). Secrets are never returned, only redacted meta (ADR-U-010).
  const [connections, setConnections] = useState<CredentialMeta[]>([]);
  const [connError, setConnError] = useState<string | null>(null);
  const [connLoaded, setConnLoaded] = useState(false);

  async function loadConnections() {
    setConnError(null);
    try {
      setConnections(await listCredentials());
      setConnLoaded(true);
    } catch (e) {
      setConnError(String(e));
    }
  }

  // Load linked connections on mount; the session passphrase is already held natively.
  useEffect(() => {
    let cancelled = false;
    listCredentials()
      .then((creds) => { if (!cancelled) { setConnections(creds); setConnLoaded(true); } })
      .catch((e) => { if (!cancelled) setConnError(String(e)); });
    return () => { cancelled = true; };
  }, []);

  async function removeConnection(kind: CredentialMeta["kind"]) {
    try {
      await disconnectCredential(kind);
      await loadConnections();
    } catch (e) {
      setConnError(String(e));
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
          onClick={async () => { if (confirm("Sign out? Your mnemonic backup is required to restore.")) { await lockSession(); clearIdentity(); } }}
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
            <button className="btn-ghost" style={{ padding: "0.2rem 0.5rem", fontSize: "0.75rem" }}
              onClick={() => saveRelays(relays.filter((x) => x !== r))}>
              Remove
            </button>
          </div>
        ))}
        <form onSubmit={addRelay} style={{ display: "flex", gap: "0.5rem", marginTop: "0.75rem" }}>
          <input placeholder="wss://your-relay.com" value={newRelay}
            onChange={(e) => setNewRelay(e.target.value)} style={{ flex: 1, fontSize: "0.875rem" }} />
          <button type="submit" className="btn-primary" style={{ width: "auto", padding: "0.6rem 1rem" }}>Add</button>
        </form>
      </section>

      <section className="card" style={{ marginBottom: "1.25rem" }}>
        <h3 style={{ margin: "0 0 0.75rem" }}>Connections</h3>
        <div className="muted" style={{ fontSize: "0.8rem", marginBottom: "0.75rem" }}>
          Your linked wallets and identities. Secrets stay encrypted on this device and are
          never shown — only their type and capabilities.
        </div>
        {connError && <div style={{ color: "var(--danger)", fontSize: "0.8rem", marginTop: "0.5rem" }}>{connError}</div>}
        {connLoaded && connections.length === 0 && (
          <div className="muted" style={{ fontSize: "0.8rem", textAlign: "center", padding: "1rem" }}>No connections linked.</div>
        )}
        {connections.length > 0 && (
          <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem", marginTop: "0.75rem" }}>
            {connections.map((c) => (
              <div key={c.kind} style={{ display: "flex", justifyContent: "space-between", alignItems: "center", padding: "0.6rem 0.75rem", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "0.4rem" }}>
                <div>
                  <div style={{ fontSize: "0.85rem" }}>{c.label}</div>
                  <div className="muted" style={{ fontSize: "0.7rem", marginTop: "0.2rem" }}>
                    {c.kind.toUpperCase()} · {c.spend_capable ? "spend" : c.receive_capable ? "receive" : "identity"}
                  </div>
                </div>
                <button className="btn-ghost" style={{ padding: "0.2rem 0.5rem", fontSize: "0.75rem", color: "var(--danger)" }}
                  onClick={() => removeConnection(c.kind)}>Disconnect</button>
              </div>
            ))}
          </div>
        )}
      </section>

      <section className="card" style={{ marginBottom: "1.25rem" }}>
        <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "0.75rem" }}>
          <h3 style={{ margin: 0 }}>Delegations</h3>
          <button className="btn-primary" style={{ width: "auto", padding: "0.4rem 0.8rem", fontSize: "0.8rem" }}
            onClick={() => setShowDelegationForm(!showDelegationForm)}>
            {showDelegationForm ? "Cancel" : "+ New"}
          </button>
        </div>

        <div className="muted" style={{ fontSize: "0.8rem", marginBottom: "0.75rem" }}>
          Delegate spend authority to child wallets or automated agents.
        </div>

        {showDelegationForm && (
          <form onSubmit={handleCreateDelegation} style={{ background: "rgba(0,0,0,0.05)", padding: "1rem", borderRadius: "0.5rem", marginBottom: "1rem", display: "flex", flexDirection: "column", gap: "0.75rem" }}>
            <div>
              <label className="muted" style={{ fontSize: "0.75rem", display: "block", marginBottom: "0.25rem" }}>Child npub</label>
              <input value={delChildNpub} onChange={e => setDelChildNpub(e.target.value)} placeholder="npub1..." style={{ width: "100%" }} required />
            </div>
            <div>
              <label className="muted" style={{ fontSize: "0.75rem", display: "block", marginBottom: "0.25rem" }}>Max per tx (sats)</label>
              <input type="number" value={delAmount} onChange={e => setDelAmount(e.target.value)} style={{ width: "100%" }} required />
            </div>
            <button type="submit" className="btn-primary" style={{ marginTop: "0.25rem" }}>Create delegation token</button>
          </form>
        )}

        {delegations.length === 0 ? (
          <div className="muted" style={{ fontSize: "0.8rem", textAlign: "center", padding: "1rem" }}>No active delegations.</div>
        ) : (
          <div style={{ display: "flex", flexDirection: "column", gap: "0.5rem" }}>
            {delegations.map(d => (
              <div key={d.token_id} style={{ fontSize: "0.8rem", padding: "0.75rem", border: "1px solid rgba(255,255,255,0.1)", borderRadius: "0.4rem" }}>
                <div style={{ display: "flex", justifyContent: "space-between" }}>
                  <span className="mono">{d.child_npub.slice(0, 12)}...</span>
                  <span style={{ color: "var(--success)" }}>Active</span>
                </div>
                <div className="muted" style={{ fontSize: "0.7rem", marginTop: "0.25rem" }}>Limit: {d.policy.max_per_tx_sats} sats/tx</div>
              </div>
            ))}
          </div>
        )}
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
