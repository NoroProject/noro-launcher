-- 0050_wave3_freezes_and_reports.sql

CREATE TABLE IF NOT EXISTS player_freezes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    actor_id UUID REFERENCES users(id) ON DELETE SET NULL,
    reason TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    released_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_player_freezes_user ON player_freezes(user_id);
CREATE INDEX IF NOT EXISTS idx_player_freezes_active ON player_freezes(user_id) WHERE released_at IS NULL;

CREATE TABLE IF NOT EXISTS player_reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_server_id UUID NOT NULL REFERENCES game_servers(id) ON DELETE CASCADE,
    reason TEXT NOT NULL,
    world TEXT,
    x DOUBLE PRECISION,
    y DOUBLE PRECISION,
    z DOUBLE PRECISION,
    status TEXT NOT NULL DEFAULT 'open',
    claimed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    claimed_at TIMESTAMPTZ,
    resolution TEXT,
    punishment_id UUID REFERENCES punishments(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_player_reports_status ON player_reports(status);
CREATE INDEX IF NOT EXISTS idx_player_reports_created ON player_reports(created_at DESC);

CREATE TABLE IF NOT EXISTS pending_report_feedbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    target_username TEXT NOT NULL,
    resolution TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_pending_report_feedbacks_user ON pending_report_feedbacks(user_id);
