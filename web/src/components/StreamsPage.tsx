import { useState } from "react";

interface StreamForm {
  recipientNpub: string;
  msatsPerPeriod: string;
  periodSeconds: string;
  preferStableChannel: boolean;
  memo: string;
}

const DEFAULT_FORM: StreamForm = {
  recipientNpub: "",
  msatsPerPeriod: "10000",
  periodSeconds: "3600",
  preferStableChannel: true,
  memo: "",
};

export default function StreamsPage() {
  const [streams] = useState<unknown[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<StreamForm>(DEFAULT_FORM);

  function handleChange(field: keyof StreamForm, value: string | boolean) {
    setForm((f) => ({ ...f, [field]: value }));
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    // Phase A6: call wasm to create stream, publish kind-30901 to Nostr
    alert("Stream creation available in Phase A6 (scheduler integration).");
    setShowForm(false);
  }

  return (
    <div className="page">
      <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", marginBottom: "1.25rem" }}>
        <h1 className="page-title" style={{ margin: 0 }}>Streams</h1>
        <button className="btn-primary" style={{ width: "auto", padding: "0.5rem 1rem" }} onClick={() => setShowForm(true)}>
          + New stream
        </button>
      </div>

      {streams.length === 0 && !showForm && (
        <div className="card muted">No active streams yet. Create one to start streaming sats.</div>
      )}

      {showForm && (
        <form className="card" onSubmit={handleSubmit} style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
          <h3 style={{ margin: 0 }}>New streaming flow</h3>

          <label>
            <div className="muted" style={{ marginBottom: "0.25rem" }}>Recipient npub</div>
            <input
              placeholder="npub1…"
              value={form.recipientNpub}
              onChange={(e) => handleChange("recipientNpub", e.target.value)}
              required
            />
          </label>

          <div style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.75rem" }}>
            <label>
              <div className="muted" style={{ marginBottom: "0.25rem" }}>Amount (msats)</div>
              <input type="number" min="1000" value={form.msatsPerPeriod}
                onChange={(e) => handleChange("msatsPerPeriod", e.target.value)} />
            </label>
            <label>
              <div className="muted" style={{ marginBottom: "0.25rem" }}>Period (seconds)</div>
              <input type="number" min="60" value={form.periodSeconds}
                onChange={(e) => handleChange("periodSeconds", e.target.value)} />
            </label>
          </div>

          <label style={{ display: "flex", gap: "0.5rem", alignItems: "center" }}>
            <input type="checkbox" checked={form.preferStableChannel}
              onChange={(e) => handleChange("preferStableChannel", e.target.checked)} />
            <span className="muted">Credit via Stable-Channel (preferred)</span>
          </label>

          <label>
            <div className="muted" style={{ marginBottom: "0.25rem" }}>Memo (optional)</div>
            <input placeholder="e.g. Monthly support" value={form.memo}
              onChange={(e) => handleChange("memo", e.target.value)} />
          </label>

          <div style={{ display: "flex", gap: "0.75rem" }}>
            <button type="submit" className="btn-primary">Create stream</button>
            <button type="button" className="btn-ghost" onClick={() => setShowForm(false)}>Cancel</button>
          </div>
        </form>
      )}
    </div>
  );
}
