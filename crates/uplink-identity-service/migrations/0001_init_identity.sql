-- Phase 5b — hosted vanity identity (receive-only). ADR-U-010 §3, ADR-U-011 §7.
--
-- Receive-only by construction: this schema stores non-secret routing plus an OPTIONAL
-- encrypted receive credential. The identity backend can never spend (no spend keys, ever).
-- Additive and migration-managed; the shared backend applies it alongside the Phase 6
-- attendance tables on the same Postgres instance.

CREATE TABLE IF NOT EXISTS identities (
    username                TEXT PRIMARY KEY,   -- normalized: lowercase, LUD-16-safe local part
    pubkey_hex              TEXT NOT NULL,       -- 64-char lowercase-hex Nostr public key
    routing_kind            TEXT NOT NULL        -- non-secret receive routing discriminant
        CHECK (routing_kind IN ('lightning_address', 'lnurl')),
    routing_address         TEXT NOT NULL,       -- 'user@domain' or 'lnurl1…' (non-secret)
    receive_credential_enc  BYTEA,               -- OPTIONAL encrypted receive-only credential
    created_at              BIGINT NOT NULL,     -- unix seconds
    revoked_at              BIGINT               -- NULL = live; set = revoked
);

-- Reverse lookup by pubkey (e.g. profile → vanity address).
CREATE INDEX IF NOT EXISTS identities_pubkey_idx ON identities (pubkey_hex);
