-- Дело: разбор жалобы целиком, а не строка в очереди.
--
-- Репорт отвечает «кто на кого пожаловался», но разбор состоит из другого:
-- что человек писал в чате, куда телепортировался модератор, что нашлось в
-- инвентаре, чем всё кончилось. Раньше это жило в памяти модератора, поэтому
-- апелляция через месяц упиралась в «я помню, он читерил».
--
-- Дело заводится на **игрока и сервер**, а не на жалобу: семь жалоб на одного
-- читера — один разбор. Частичный уникальный индекс ниже и есть склейка.

CREATE TABLE IF NOT EXISTS cases (
    id             UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    target_id      UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    game_server_id UUID REFERENCES game_servers(id) ON DELETE SET NULL,
    -- open → in_review (кто-то взял) → resolved | rejected.
    status         TEXT NOT NULL DEFAULT 'open'
                   CHECK (status IN ('open', 'in_review', 'resolved', 'rejected')),
    claimed_by     UUID REFERENCES users(id) ON DELETE SET NULL,
    claimed_at     TIMESTAMPTZ,
    opened_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at    TIMESTAMPTZ,
    -- Чем кончилось: подтвердилось, не подтвердилось, не хватило данных.
    verdict        TEXT CHECK (verdict IN ('confirmed', 'rejected', 'insufficient')),
    -- Пункт свода и его снимок: правило переименуют, а разбор должен читаться.
    rule_id        UUID REFERENCES rules(id) ON DELETE SET NULL,
    rule_code      TEXT,
    resolution     TEXT NOT NULL DEFAULT ''
);

-- Открытое дело на игрока и сервер ровно одно: следующая жалоба попадает в
-- него, а не заводит второе. Закрытых дел на того же игрока сколько угодно —
-- условие индекса их не трогает.
CREATE UNIQUE INDEX IF NOT EXISTS idx_cases_open_target
    ON cases (target_id, game_server_id) WHERE status IN ('open', 'in_review');
CREATE INDEX IF NOT EXISTS idx_cases_status ON cases (status, opened_at DESC);
CREATE INDEX IF NOT EXISTS idx_cases_target ON cases (target_id);

ALTER TABLE player_reports ADD COLUMN IF NOT EXISTS case_id UUID REFERENCES cases(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_player_reports_case ON player_reports (case_id);

-- Наказание ссылается на дело, а не дело на наказание: за один разбор их
-- бывает несколько — мут за чат и бан за чит.
ALTER TABLE punishments ADD COLUMN IF NOT EXISTS case_id UUID REFERENCES cases(id) ON DELETE SET NULL;
CREATE INDEX IF NOT EXISTS idx_punishments_case ON punishments (case_id);

-- Лента разбора. `source` обязателен: по ленте должно быть видно, сделали это
-- с сайта или из игры, иначе спорные разборы нечем восстановить.
CREATE TABLE IF NOT EXISTS case_events (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id     UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actor_id    UUID REFERENCES users(id) ON DELETE SET NULL,
    -- Снимок имени: аккаунт модератора может исчезнуть, событие остаётся.
    actor_label TEXT NOT NULL DEFAULT '',
    source      TEXT NOT NULL DEFAULT 'system' CHECK (source IN ('web', 'game', 'system')),
    kind        TEXT NOT NULL,
    payload     JSONB NOT NULL DEFAULT '{}'::jsonb
);
CREATE INDEX IF NOT EXISTS idx_case_events_case ON case_events (case_id, at);

-- Срез чата вокруг события. Общий чат-лог не ведётся: агент держит последние
-- сообщения в памяти и отдаёт окно, только когда появился повод.
--
-- Команды аутентификации (`/login`, `/register`, …) сюда не попадают вовсе —
-- фильтр стоит в агенте, до отправки: пароль игрока не должен доехать даже до
-- канала, не то что до глаз модератора.
CREATE TABLE IF NOT EXISTS case_messages (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    case_id     UUID NOT NULL REFERENCES cases(id) ON DELETE CASCADE,
    at          TIMESTAMPTZ NOT NULL,
    sender_id   UUID REFERENCES users(id) ON DELETE SET NULL,
    sender_name TEXT NOT NULL,
    channel     TEXT NOT NULL CHECK (channel IN ('public', 'local', 'private', 'command')),
    content     TEXT NOT NULL
);
CREATE INDEX IF NOT EXISTS idx_case_messages_case ON case_messages (case_id, at);

-- Открытые жалобы, что уже лежат в очереди, разносятся по делам: иначе
-- страница дел на живой базе откроется пустой, а очередь репортов — полной.
DO $$
DECLARE
    r RECORD;
    target_case UUID;
BEGIN
    FOR r IN
        SELECT id, target_id, game_server_id, created_at
          FROM player_reports
         WHERE case_id IS NULL AND status IN ('open', 'claimed')
         ORDER BY created_at
    LOOP
        SELECT id INTO target_case FROM cases
         WHERE target_id = r.target_id
           AND game_server_id IS NOT DISTINCT FROM r.game_server_id
           AND status IN ('open', 'in_review');

        IF target_case IS NULL THEN
            INSERT INTO cases (target_id, game_server_id, opened_at)
            VALUES (r.target_id, r.game_server_id, r.created_at)
            RETURNING id INTO target_case;
        END IF;

        UPDATE player_reports SET case_id = target_case WHERE id = r.id;
    END LOOP;
END $$;
