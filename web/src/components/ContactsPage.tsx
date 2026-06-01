import { useState } from "react";

interface Contact {
  npub: string;
  displayName?: string;
  lud16?: string;
}

export default function ContactsPage() {
  const [contacts, setContacts] = useState<Contact[]>([]);
  const [input, setInput] = useState("");
  const [loading, setLoading] = useState(false);

  async function handleAdd(e: React.FormEvent) {
    e.preventDefault();
    if (!input.trim()) return;
    setLoading(true);
    // Phase A2: resolve kind-0 profile from Nostr relay via wasm
    // For now, add as a bare npub
    setContacts((c) => [...c, { npub: input.trim() }]);
    setInput("");
    setLoading(false);
  }

  return (
    <div className="page">
      <h1 className="page-title">Contacts</h1>

      <form onSubmit={handleAdd} style={{ display: "flex", gap: "0.5rem", marginBottom: "1.25rem" }}>
        <input
          placeholder="npub1…"
          value={input}
          onChange={(e) => setInput(e.target.value)}
          style={{ flex: 1 }}
        />
        <button type="submit" className="btn-primary" style={{ width: "auto", padding: "0.75rem 1.25rem" }} disabled={loading}>
          Add
        </button>
      </form>

      {contacts.length === 0 && (
        <div className="card muted">No contacts yet. Add a recipient's npub to start streaming sats to them.</div>
      )}

      {contacts.map((c) => (
        <div key={c.npub} className="card" style={{ marginBottom: "0.75rem", display: "flex", justifyContent: "space-between" }}>
          <div>
            <div className="mono">{c.npub.slice(0, 16)}…{c.npub.slice(-8)}</div>
            {c.lud16 && <div className="muted">{c.lud16}</div>}
          </div>
          <button
            className="btn-ghost"
            style={{ padding: "0.4rem 0.75rem", fontSize: "0.875rem" }}
            onClick={() => {/* Phase A5: navigate to stream creation with this contact */}}
          >
            Stream ⚡
          </button>
        </div>
      ))}
    </div>
  );
}
