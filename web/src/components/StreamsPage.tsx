import { useState } from "react";
import { upsertStream, removeStream, publishStreamDeclaration } from "../wasm/uplink-client.ts";

interface StreamPolicy {
  stream_id: string;
  recipient_npub_hex: string;
  msats_per_period: number;
  period_seconds: number;
  status: string;
}

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
  const [streams, setStreams] = useState<StreamPolicy[]>([]);
  const [showForm, setShowForm] = useState(false);
  const [form, setForm] = useState<StreamForm>(DEFAULT_FORM);

  function handleChange(field: keyof StreamForm, value: string | boolean) {
    setForm((f) => ({ ...f, [field]: value }));
  }

  async function handleSubmit(e: React.FormEvent) {
    e.preventDefault();
    try {
      const streamId = `str-${Math.random().toString(16).slice(2, 10)}`;
      const now = Math.floor(Date.now() / 1000);

      // 1. Add to local scheduler
      await upsertStream({
        streamId,
        recipientNpub: form.recipientNpub,
        msatsPerPeriod: parseInt(form.msatsPerPeriod),
        periodSeconds: parseInt(form.periodSeconds),
        startAtUnix: now,
      });

      // 2. Publish to Nostr
      await publishStreamDeclaration(
        streamId,
        form.recipientNpub,
        parseInt(form.msatsPerPeriod),
        parseInt(form.periodSeconds),
        now
      );

      setStreams([...streams, {
        stream_id: streamId,
        recipient_npub_hex: form.recipientNpub,
        msats_per_period: parseInt(form.msatsPerPeriod),
        period_seconds: parseInt(form.periodSeconds),
        status: "Active"
      }]);

      setShowForm(false);
      setForm(DEFAULT_FORM);
      alert("Stream created and published to Nostr!");
    } catch (err: any) {
      alert(`Error creating stream: ${err.message || err}`);
    }
  }

  async function handleDelete(streamId: string) {
    if (confirm("Stop this stream?")) {
      await removeStream(streamId);
      setStreams(streams.filter(s => s.stream_id !== streamId));
    }
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

      {!showForm && streams.length > 0 && (
        <div style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
          {streams.map((s) => (
            <div key={s.stream_id} className="card" style={{ display: "flex", justifyContent: "space-between", alignItems: "center" }}>
              <div>
                <div style={{ fontWeight: 600 }}>{s.msats_per_period / 1000} sats every {s.period_seconds}s</div>
                <div className="muted" style={{ fontSize: "0.8rem", marginTop: "0.25rem" }}>
                  To: {s.recipient_npub_hex.slice(0, 12)}...
                </div>
              </div>
              <button className="btn-ghost" style={{ color: "var(--danger)", padding: "0.4rem 0.8rem", fontSize: "0.8rem" }}
                onClick={() => handleDelete(s.stream_id)}>
                Stop
              </button>
            </div>
          ))}
        </div>
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
