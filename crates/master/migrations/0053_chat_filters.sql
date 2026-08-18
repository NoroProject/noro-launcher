-- 0053_chat_filters.sql

CREATE TABLE IF NOT EXISTS chat_filters (
    filter_type TEXT PRIMARY KEY,
    mode TEXT NOT NULL DEFAULT 'deny',
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    config JSONB NOT NULL DEFAULT '{}'::jsonb
);

INSERT INTO chat_filters (filter_type, mode, enabled, config)
VALUES
    ('ad', 'punish', true, '{"whitelist":["noro.dalynkaa.dev","dalynkaa.dev"]}'::jsonb),
    ('word', 'deny', true, '{"words":["badword"]}'::jsonb),
    ('caps', 'deny', true, '{"threshold":0.6,"min_length":6}'::jsonb),
    ('flood', 'deny', true, '{"max_messages":3,"window_secs":4}'::jsonb)
ON CONFLICT (filter_type) DO NOTHING;
