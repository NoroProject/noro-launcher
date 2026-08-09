-- Таблица доступа игроков к конкретным плащам из каталога
CREATE TABLE IF NOT EXISTS user_capes (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cape_id UUID NOT NULL REFERENCES capes(id) ON DELETE CASCADE,
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, cape_id)
);
