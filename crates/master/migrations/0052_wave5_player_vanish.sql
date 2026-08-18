-- Состояние невидимости (ваниш) пользователей.
-- Переживает рестарт сервера и хранится на мастере.

CREATE TABLE IF NOT EXISTS player_vanish (
    user_id UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Флаг "войти тихо" (автованиш при входе), доступный для модераторов с правом noro.mod.vanish.silent_join / noro.mod.vanish.use.
ALTER TABLE users ADD COLUMN IF NOT EXISTS silent_join BOOLEAN NOT NULL DEFAULT FALSE;
