-- Player requests to add an optional mod to a build.
CREATE TABLE IF NOT EXISTS mod_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    server_id UUID NOT NULL REFERENCES servers(id) ON DELETE CASCADE,
    build_id UUID REFERENCES builds(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    project_id TEXT NOT NULL,
    title TEXT NOT NULL,
    icon_url TEXT,
    description TEXT,
    suggested_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_mod_suggestions_server ON mod_suggestions(server_id);
CREATE INDEX IF NOT EXISTS idx_mod_suggestions_status ON mod_suggestions(status);
