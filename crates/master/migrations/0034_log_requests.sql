-- Запросы логов у игрока.
--
-- Два режима, и разница между ними не в удобстве. Согласие уместно в поддержке
-- и снижает тревогу; в расследовании оно бессмысленно — единственный, чьи логи
-- никогда не придут, это тот, ради кого всё затевалось. Поэтому принудительный
-- режим существует, но живёт за отдельным правом и обязательной причиной.
CREATE TABLE IF NOT EXISTS log_requests (
    id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    actor_label TEXT NOT NULL,
    target_id  UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    -- Обязательна: игрок видит её в модалке, и она же остаётся в аудите.
    reason     TEXT NOT NULL,
    server_id  UUID,
    -- Без спроса. Каждое применение — отдельное событие в audit_log.
    forced     BOOLEAN NOT NULL DEFAULT FALSE,

    -- pending | accepted | declined | expired | delivered
    status     TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Пять минут: столько живёт модалка у игрока.
    expires_at TIMESTAMPTZ NOT NULL,
    answered_at TIMESTAMPTZ,
    bundle_id  UUID REFERENCES support_bundles(id) ON DELETE SET NULL
);

CREATE INDEX IF NOT EXISTS log_requests_target_idx ON log_requests (target_id, created_at DESC);
CREATE INDEX IF NOT EXISTS log_requests_open_idx ON log_requests (expires_at)
    WHERE status = 'pending';
