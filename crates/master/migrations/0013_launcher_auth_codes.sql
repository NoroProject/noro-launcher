-- One-time codes for the launcher login handoff. The player's browser hits the
-- launcher's loopback port with a code, and the launcher trades it for tokens
-- over HTTPS. Only the code travels in a URL, so tokens stay out of browser
-- history and proxy logs — and the master never has to reach the player's
-- machine, which it cannot do from inside a container anyway.
CREATE TABLE IF NOT EXISTS launcher_auth_codes (
    code          UUID PRIMARY KEY,
    user_id       UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    access_token  UUID NOT NULL,
    refresh_token UUID NOT NULL,
    expires_at    TIMESTAMPTZ NOT NULL
);

CREATE INDEX IF NOT EXISTS launcher_auth_codes_expires ON launcher_auth_codes (expires_at);
