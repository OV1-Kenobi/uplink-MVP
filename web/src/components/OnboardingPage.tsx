import { useState, useEffect } from "react";
import { createIdentity, restoreIdentity, exportMnemonicWords, unlockIdentity, hasIdentity, resetIdentity, setLightningAddress, connectNwc, connectLnc, linkIdentity } from "../identity.ts";
import { useIdentityStore } from "../store/identityStore.ts";

type Step = "welcome" | "address" | "connect" | "link" | "generate" | "backup" | "restore" | "unlock";

export default function OnboardingPage() {
  const setIdentity = useIdentityStore((s) => s.setIdentity);
  const [step, setStep] = useState<Step>("welcome");
  const [words, setWords] = useState<string[]>([]);
  const [pendingNpub, setPendingNpub] = useState<string | null>(null);
  const [restorePhrase, setRestorePhrase] = useState("");
  const [password, setPassword] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  // Phase 5a — bring-your-own-credential inputs (ADR-U-010)
  const [lnAddress, setLnAddress] = useState("");
  const [nwcUri, setNwcUri] = useState("");
  const [lncPhrase, setLncPhrase] = useState("");
  const [identityValue, setIdentityValue] = useState("");
  const [connectMode, setConnectMode] = useState<"nwc" | "lnc">("nwc");
  const [linkMode, setLinkMode] = useState<"npub" | "nip05">("npub");

  useEffect(() => {
    // If an identity already exists in storage, show the unlock screen.
    // Routed through the facade so it works on both the native (Tauri/sled)
    // and wasm (browser) targets.
    let cancelled = false;
    hasIdentity()
      .then((exists) => {
        if (!cancelled && exists) setStep("unlock");
      })
      .catch(() => {});
    return () => {
      cancelled = true;
    };
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

  // Bring-your-own-credential: silently provision a local signing identity (no forced
  // backup screen), persist the external credential, then enter the app. The device
  // password is the encryption key for credentials at rest (ADR-U-010).
  async function provisionWithCredential(store: () => Promise<unknown>) {
    if (password.length < 8) {
      setError("Password must be at least 8 characters");
      return;
    }
    setLoading(true);
    setError(null);
    try {
      const npub = await createIdentity(password, 0);
      await store();
      setIdentity(npub, 0);
    } catch (e) {
      setError(String(e));
    } finally {
      setLoading(false);
    }
  }

  const handleUseAddress = () =>
    provisionWithCredential(() => setLightningAddress(lnAddress.trim()));
  const handleConnectNwc = () =>
    provisionWithCredential(() => connectNwc(nwcUri.trim()));
  const handleConnectLnc = () =>
    provisionWithCredential(() => connectLnc(lncPhrase.trim()));
  const handleLinkIdentity = () =>
    provisionWithCredential(() => linkIdentity(linkMode, identityValue.trim()));

  if (step === "welcome") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "3rem" }}>
        <h1 style={{ fontSize: "2rem", margin: 0 }}>⚡ Uplink</h1>
        <p className="muted">Streaming sats to your existing wallet — bring what you already have.</p>

        <button className="btn-primary" onClick={() => { setError(null); setStep("address"); }}>
          Use my Lightning Address
        </button>
        <p className="muted small" style={{ marginTop: "-0.5rem" }}>
          Receive straight to your own wallet. No custody, no setup.
        </p>

        <button className="btn-ghost" onClick={() => { setError(null); setStep("connect"); }}>
          Connect a wallet (NWC / Lightning Node Connect)
        </button>
        <button className="btn-ghost" onClick={() => { setError(null); setStep("link"); }}>
          Link a Nostr identity (npub / NIP-05)
        </button>

        <hr style={{ width: "100%", border: "none", borderTop: "1px solid var(--surface-2)", margin: "0.5rem 0" }} />
        <button className="btn-ghost" onClick={() => { setError(null); setStep("generate"); }}>Create new identity</button>
        <button className="btn-ghost" onClick={() => { setError(null); setStep("restore"); }}>Restore from mnemonic</button>
      </div>
    );
  }

  if (step === "address") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "3rem" }}>
        <h2>Use your Lightning Address</h2>
        <p className="muted">Sats land directly in the wallet behind this address.</p>
        <input placeholder="you@wallet.com" value={lnAddress} onChange={(e) => setLnAddress(e.target.value)} />
        <input type="password" placeholder="Set a device password (min 8 chars)…" value={password} onChange={(e) => setPassword(e.target.value)} />
        <p className="muted small">Encrypts your connections on this device.</p>
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={handleUseAddress} disabled={loading || !lnAddress.trim() || password.length < 8}>
          {loading ? "Setting up…" : "Continue"}
        </button>
        <button className="btn-ghost" onClick={() => setStep("welcome")}>← Back</button>
      </div>
    );
  }

  if (step === "connect") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "3rem" }}>
        <h2>Connect a wallet</h2>
        <div style={{ display: "flex", gap: "0.5rem" }}>
          <button className={connectMode === "nwc" ? "btn-primary" : "btn-ghost"} style={{ flex: 1 }} onClick={() => setConnectMode("nwc")}>NWC (spend + receive)</button>
          <button className={connectMode === "lnc" ? "btn-primary" : "btn-ghost"} style={{ flex: 1 }} onClick={() => setConnectMode("lnc")}>LNC (soon)</button>
        </div>
        {connectMode === "nwc" ? (
          <>
            <p className="muted small">Nostr Wallet Connect — receive &amp; spend. Works with AlbyHub, Breez, and LND (via Alby).</p>
            <textarea rows={3} placeholder="nostr+walletconnect://…" value={nwcUri} onChange={(e) => setNwcUri(e.target.value)} />
          </>
        ) : (
          <>
            <p className="muted small">Lightning Node Connect — direct LND spend. Saved and capability-flagged now; spending activates once the secure transport ships (use NWC to spend today).</p>
            <textarea rows={3} placeholder="10-word pairing phrase…" value={lncPhrase} onChange={(e) => setLncPhrase(e.target.value)} />
          </>
        )}
        <input type="password" placeholder="Set a device password (min 8 chars)…" value={password} onChange={(e) => setPassword(e.target.value)} />
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={connectMode === "nwc" ? handleConnectNwc : handleConnectLnc}
          disabled={loading || password.length < 8 || (connectMode === "nwc" ? !nwcUri.trim() : !lncPhrase.trim())}>
          {loading ? "Connecting…" : "Connect"}
        </button>
        <button className="btn-ghost" onClick={() => setStep("welcome")}>← Back</button>
      </div>
    );
  }

  if (step === "link") {
    return (
      <div className="page" style={{ display: "flex", flexDirection: "column", gap: "1rem", paddingTop: "3rem" }}>
        <h2>Link a Nostr identity</h2>
        <div style={{ display: "flex", gap: "0.5rem" }}>
          <button className={linkMode === "npub" ? "btn-primary" : "btn-ghost"} style={{ flex: 1 }} onClick={() => setLinkMode("npub")}>npub</button>
          <button className={linkMode === "nip05" ? "btn-primary" : "btn-ghost"} style={{ flex: 1 }} onClick={() => setLinkMode("nip05")}>NIP-05</button>
        </div>
        <input placeholder={linkMode === "npub" ? "npub1…" : "you@domain.com"} value={identityValue} onChange={(e) => setIdentityValue(e.target.value)} />
        <input type="password" placeholder="Set a device password (min 8 chars)…" value={password} onChange={(e) => setPassword(e.target.value)} />
        {error && <div style={{ color: "var(--danger)" }}>{error}</div>}
        <button className="btn-primary" onClick={handleLinkIdentity} disabled={loading || !identityValue.trim() || password.length < 8}>
          {loading ? "Linking…" : "Continue"}
        </button>
        <button className="btn-ghost" onClick={() => setStep("welcome")}>← Back</button>
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
          Lost password? <button className="btn-link" onClick={async () => { await resetIdentity(); window.location.reload(); }}>Reset app</button>
        </p>
      </div>
    );
  }

  return null;
}
