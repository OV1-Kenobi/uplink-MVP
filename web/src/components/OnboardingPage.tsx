import { useState } from "react";
import { createIdentity, restoreIdentity, exportMnemonicWords } from "../wasm/uplink-client.ts";
import { useIdentityStore } from "../store/identityStore.ts";

type Step = "welcome" | "generate" | "backup" | "restore" | "confirm";

export default function OnboardingPage() {
  const setIdentity = useIdentityStore((s) => s.setIdentity);
  const [step, setStep] = useState<Step>("welcome");
  const [words, setWords] = useState<string[]>([]);
  const [pendingNpub, setPendingNpub] = useState<string | null>(null);
  const [restorePhrase, setRestorePhrase] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  async function handleGenerate() {
    setLoading(true);
    setError(null);
    try {
      const npub = await createIdentity(0);
      const mnemonic = await exportMnemonicWords();
      setWords(mnemonic);
      setPendingNpub(npub); // hold npub until user confirms backup
      setStep("backup");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleRestore() {
    setLoading(true);
    setError(null);
    try {
      const npub = await restoreIdentity(restorePhrase.trim(), 0);
      setIdentity(npub, 0);
      // App.tsx will re-render and show the dashboard
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  if (step === "welcome") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "4rem" }}>
        <h1 style={{ fontSize: "2rem", margin: 0 }}>⚡ Uplink</h1>
        <p className="muted">Nostr-native streaming sats with Stable-Channel wallets.</p>
        <button className="btn-primary" onClick={() => setStep("generate")}>Create new identity</button>
        <button className="btn-ghost" onClick={() => setStep("restore")}>Restore from mnemonic</button>
      </div>
    );
  }

  if (step === "generate") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "3rem" }}>
        <h2>New identity</h2>
        <p className="muted">We'll generate a 12-word mnemonic. Write it down — it's your only backup.</p>
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={handleGenerate} disabled={loading}>
          {loading ? "Generating…" : "Generate mnemonic"}
        </button>
        <button className="btn-ghost" onClick={() => setStep("welcome")}>← Back</button>
      </div>
    );
  }

  if (step === "backup") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem" }}>
        <h2>⚠️ Back up your mnemonic</h2>
        <p className="muted">Write these 12 words on paper and store them offline. This is shown once.</p>
        <div className="card" style={{ display: "grid", gridTemplateColumns: "1fr 1fr", gap: "0.5rem" }}>
          {words.map((word, i) => (
            <div key={i} className="mono" style={{ padding: "0.4rem 0.6rem", background: "var(--surface-2)", borderRadius: "6px" }}>
              <span className="muted">{i + 1}. </span>{word}
            </div>
          ))}
        </div>
        <button className="btn-primary" onClick={() => { if (pendingNpub) setIdentity(pendingNpub, 0); }}>
          I've written it down →
        </button>
      </div>
    );
  }

  if (step === "restore") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "3rem" }}>
        <h2>Restore identity</h2>
        <textarea
          rows={4}
          placeholder="Enter your 12 or 24 word mnemonic phrase…"
          value={restorePhrase}
          onChange={(e) => setRestorePhrase(e.target.value)}
        />
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={handleRestore} disabled={loading || !restorePhrase.trim()}>
          {loading ? "Restoring…" : "Restore"}
        </button>
        <button className="btn-ghost" onClick={() => setStep("welcome")}>← Back</button>
      </div>
    );
  }

  return null;
}
