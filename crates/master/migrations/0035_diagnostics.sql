-- Последняя диагностика лаунчера. Одна строка на игрока: важно текущее
-- состояние, а история версий и свободного места никому не нужна.
CREATE TABLE IF NOT EXISTS launcher_diagnostics (
    user_id  UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    report   JSONB NOT NULL
);
