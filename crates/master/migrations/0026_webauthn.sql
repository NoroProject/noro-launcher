-- Real WebAuthn verification, replacing a login that trusted credential_id.
--
-- credential_id is a public value the browser hands over, and the old flow
-- issued a session as soon as it matched a row — no signature check at all.
-- Existing rows can't be upgraded (the stored public_key has no defined format
-- and nothing was bound to a challenge), and keeping them would keep a path
-- around the new check. Everyone still has Discord login, so nobody is locked
-- out by this.
DELETE FROM passkeys;

ALTER TABLE passkeys DROP COLUMN IF EXISTS public_key;
ALTER TABLE passkeys DROP COLUMN IF EXISTS counter;

-- The whole credential as webauthn-rs serialises it: key, counter and flags in
-- one blob. Storing the pieces separately means rebuilding the library's struct
-- by hand every time it changes.
ALTER TABLE passkeys ADD COLUMN IF NOT EXISTS credential JSONB NOT NULL;
ALTER TABLE passkeys ADD COLUMN IF NOT EXISTS last_used_at TIMESTAMPTZ;

-- State carried between /options and /verify. A bare challenge isn't enough:
-- verification also needs the policy — user verification, allowed credentials,
-- which user this was started for.
DROP TABLE IF EXISTS passkey_challenges;

CREATE TABLE IF NOT EXISTS webauthn_states (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- NULL for login: who is signing in is only known once it verifies.
    user_id    UUID REFERENCES users(id) ON DELETE CASCADE,
    kind       TEXT NOT NULL CHECK (kind IN ('register', 'login')),
    state      JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS webauthn_states_expires_idx ON webauthn_states (expires_at);
