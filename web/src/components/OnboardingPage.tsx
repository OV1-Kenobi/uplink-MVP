import { useState, useEffect } from "react";
import { createIdentity, restoreIdentity, exportMnemonicWords, unlockIdentity } from "../wasm/uplink-client.ts";
import { useIdentityStore } from "../store/identityStore.ts";

type Step = "welcome" | "generate" | "backup" | "restore" | "unlock";

export default function OnboardingPage() {
  const setIdentity = useIdentityStore((s) => s.setIdentity);
  const [step, setStep] = useState<Step>("welcome");
  const [words, setWords] = useState<string[]>([]);
  const [pendingNpub, setPendingNpub] = useState<string | null>(null);
  const [restorePhrase, setRestorePhrase] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    // If an identity already exists in storage, show unlock screen
    if (localStorage.getItem("identity_mnemonic")) {
      setStep("unlock");
    }
  }, []);

  async function handleGenerate() {
    if (password.length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const npub = await createIdentity(password, 0);
      const mnemonic = await exportMnemonicWords();
      setWords(mnemonic);
      setPendingNpub(npub);
      setStep("backup");
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleRestore() {
    if (password.length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const npub = await restoreIdentity(restorePhrase.trim(), password, 0);
      setIdentity(npub, 0);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  async function handleUnlock() {
    setLoading(true);
    setError(null);
    try {
      const npub = await unlockIdentity(password);
      setIdentity(npub, 0);
    } catch (e) {
      setError("Unlock failed. Incorrect password?");
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
        <p className="muted">Create a password to encrypt your wallet on this device.</p>
        <input
          type="password"
          placeholder="Min 8 characters…"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        <p className="muted small">We'll generate a 12-word mnemonic next. Write it down!</p>
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={handleGenerate} disabled={loading || password.length < 8}>
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
        <p className="muted">Write these 24 words on paper and store them offline. This is shown once.</p>
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
          rows={3}
          placeholder="Enter your mnemonic phrase…"
          value={restorePhrase}
          onChange={(e) => setRestorePhrase(e.target.value)}
        />
        <input
          type="password"
          placeholder="New wallet password (min 8 chars)…"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
        />
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={handleRestore} disabled={loading || !restorePhrase.trim() || password.length < 8}>
          {loading ? "Restoring…" : "Restore"}
        </button>
        <button className="btn-ghost" onClick={() => setStep("welcome")}>← Back</button>
      </div>
    );
  }

  if (step === "unlock") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "5rem" }}>
        <h1 style={{ fontSize: "2rem", margin: 0 }}>⚡ Uplink</h1>
        <h2>Unlock wallet</h2>
        <input
          type="password"
          autoFocus
          placeholder="Password…"
          value={password}
          onChange={(e) => setPassword(e.target.value)}
          onKeyDown={(e) => e.key === "Enter" && handleUnlock()}
        />
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={handleUnlock} disabled={loading || !password}>
          {loading ? "Unlocking…" : "Unlock"}
        </button>
        <p className="muted small" style={{ marginTop: "2rem" }}>
          Lost password? <button className="btn-link" onClick={() => { localStorage.clear(); window.location.reload(); }}>Reset app</button>
        </p>
      </div>
    );
  }

  return null;
}
