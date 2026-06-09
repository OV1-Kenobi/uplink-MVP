-- Phase 6 — authoritative attendance schema. ADR-U-011 §7 (five tables).
--
-- Shares the single Postgres instance with the Phase 5b identity schema
-- (crates/uplink-identity-service/migrations/0001_init_identity.sql); 5b lands first, so this
-- is migration 0002 in the shared DB. Additive and migration-managed.
--
-- Custody (ADR-U-011 §8): office tag keys and the backend writer key are bearer secrets —
-- stored ENCRYPTED at rest, never logged, never returned to any client. This backend gates
-- and audits; it can never spend.

-- 1. relay_auth_keys — NIP-42 allowlist; each pubkey carries a writer role (§2).
CREATE TABLE IF NOT EXISTS relay_auth_keys (
    pubkey       TEXT PRIMARY KEY,        -- 64-char lowercase-hex Nostr public key
    role         TEXT NOT NULL            -- writer authorization (§2-3)
        CHECK (role IN ('worker', 'backend', 'admin')),
    enrolled_at  BIGINT NOT NULL,          -- unix seconds
    revoked_at   BIGINT                    -- NULL = active
);

-- 2. office_tags — enrolled NTAG 424 tags; server-held keys + last-seen counter (§4-5).
CREATE TABLE IF NOT EXISTS office_tags (
    uid            BYTEA PRIMARY KEY,      -- 7-byte tag UID recovered by SDM verify
    office_id      TEXT NOT NULL,           -- the office this tag gates
    tag_keys_enc   BYTEA NOT NULL,          -- ENCRYPTED SDM meta/file-read keys (bearer secret)
    last_read_ctr  BIGINT,                  -- last accepted read counter (NULL = never tapped)
    enrolled_at    BIGINT NOT NULL,
    revoked_at     BIGINT                   -- NULL = active
);

-- 3. attendance_events_raw — verbatim signed events, accepted OR rejected, for audit/replay
--    (§2 retention, §5 rejects recorded raw). Idempotent on (uid, read_ctr) for accepted taps.
CREATE TABLE IF NOT EXISTS attendance_events_raw (
    event_id     TEXT PRIMARY KEY,         -- Nostr event id (content hash)
    pubkey       TEXT NOT NULL,             -- sender (worker/backend/admin) pubkey hex
    kind         INTEGER NOT NULL,          -- attendance event kind (9910/30910/9911/9912)
    created_at   BIGINT NOT NULL,           -- event-declared unix seconds
    received_at  BIGINT NOT NULL,           -- server receipt unix seconds
    raw_event    JSONB NOT NULL,            -- verbatim signed event (never mutated)
    parsed_uid   BYTEA,                     -- recovered UID (NULL if verify failed/not a tap)
    read_ctr     BIGINT,                    -- recovered read counter (NULL if not verified)
    accepted     BOOLEAN NOT NULL,          -- result of the 7-step validation
    reject_code  TEXT                       -- stable RejectReason::code() when not accepted
);

CREATE INDEX IF NOT EXISTS attendance_events_raw_pubkey_idx
    ON attendance_events_raw (pubkey);
-- One accepted tap per (uid, read_ctr): enforces step-7 idempotency / replay defense.
CREATE UNIQUE INDEX IF NOT EXISTS attendance_events_raw_uid_ctr_idx
    ON attendance_events_raw (parsed_uid, read_ctr)
    WHERE accepted AND parsed_uid IS NOT NULL;

-- 4. attendance_sessions — authoritative WorkSession per (worker, stream) (§5.6).
CREATE TABLE IF NOT EXISTS attendance_sessions (
    session_id     TEXT PRIMARY KEY,        -- = opening tap's event id (deterministic)
    worker_pubkey  TEXT NOT NULL,            -- worker pubkey hex
    stream_id      TEXT NOT NULL,            -- the in-office stream this session gates
    status         TEXT NOT NULL             -- SessionStatus
        CHECK (status IN ('open', 'closed', 'suspended', 'auto_closed')),
    opened_at      BIGINT NOT NULL,
    closed_at      BIGINT                    -- set for closed / auto_closed
);

CREATE INDEX IF NOT EXISTS attendance_sessions_worker_stream_idx
    ON attendance_sessions (worker_pubkey, stream_id);
-- At most one OPEN session per (worker, stream): single-open-per-stream invariant.
CREATE UNIQUE INDEX IF NOT EXISTS attendance_sessions_one_open_idx
    ON attendance_sessions (worker_pubkey, stream_id)
    WHERE status = 'open';

-- 5. stream_intervals — session-gated 6-minute payout intervals (§6); intent_id idempotency.
CREATE TABLE IF NOT EXISTS stream_intervals (
    session_id     TEXT NOT NULL REFERENCES attendance_sessions (session_id),
    period_index   BIGINT NOT NULL,          -- fixed IN_OFFICE_PERIOD_SECONDS interval index
    intent_id      TEXT NOT NULL,            -- 'stream_id:period_index' (ADR-U-008 §7)
    payout_status  TEXT NOT NULL             -- authorization/payment lifecycle
        CHECK (payout_status IN ('pending', 'authorized', 'paid', 'failed')),
    created_at     BIGINT NOT NULL,
    PRIMARY KEY (session_id, period_index)
);

-- One row per payout intent: the backend authorizes each interval at most once.
CREATE UNIQUE INDEX IF NOT EXISTS stream_intervals_intent_idx
    ON stream_intervals (intent_id);
