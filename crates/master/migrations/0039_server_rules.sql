-- Свод правил проекта: категории любой вложенности и сами правила.
--
-- Правило живёт либо в общем своде (server_id IS NULL), либо дополняет
-- конкретный сервер. Наследования нет: у правила свой scope, потому что иначе
-- каждый запрос из игры превращался бы в обход дерева категорий вверх.

CREATE TABLE IF NOT EXISTS rule_categories (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- SET NULL, а не CASCADE: удаление раздела не должно молча уносить
    -- подразделы вместе с их правилами — они всплывают в корень и видны.
    parent_id   UUID REFERENCES rule_categories(id) ON DELETE SET NULL,
    server_id   UUID REFERENCES servers(id) ON DELETE CASCADE,
    -- Номер раздела («1», «2.3»). Из него админка выводит коды правил.
    code        VARCHAR(32)  NOT NULL DEFAULT '',
    name        VARCHAR(255) NOT NULL,
    description TEXT         NOT NULL DEFAULT '',
    sort_order  INT          NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at  TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS rules (
    id                 UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    category_id        UUID REFERENCES rule_categories(id) ON DELETE SET NULL,
    server_id          UUID REFERENCES servers(id) ON DELETE CASCADE,
    -- Короткий код («1.1»): им ссылаются в чате, банах и тикетах.
    code               VARCHAR(32)  NOT NULL,
    title              VARCHAR(255) NOT NULL,
    description        TEXT         NOT NULL DEFAULT '',
    -- Рекомендация модератору: вид наказания и срок в минутах. Не текст:
    -- строкой вроде «бан 1-7д» нельзя ни подставить в форму, ни применить
    -- из игры — её пришлось бы разбирать обратно на каждой стороне.
    punishment_kind    TEXT CHECK (punishment_kind IN ('warn', 'mute', 'ban', 'server_ban')),
    -- NULL при заданном виде — навсегда.
    punishment_minutes BIGINT CHECK (punishment_minutes > 0),
    sort_order         INT          NOT NULL DEFAULT 0,
    created_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW(),
    updated_at         TIMESTAMPTZ  NOT NULL DEFAULT NOW()
);

-- Код обязан быть один на свод: по нему ищут правило, и два «1.1» рядом
-- означают, что ссылка в бане перестаёт что-либо доказывать. Общий свод и
-- дополнение сервера нумеруются независимо, поэтому индекса два.
CREATE UNIQUE INDEX IF NOT EXISTS idx_rules_code_global
    ON rules (lower(code)) WHERE server_id IS NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_rules_code_server
    ON rules (server_id, lower(code)) WHERE server_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_rules_server ON rules (server_id);
CREATE INDEX IF NOT EXISTS idx_rules_category ON rules (category_id);
CREATE INDEX IF NOT EXISTS idx_rule_categories_server ON rule_categories (server_id);
CREATE INDEX IF NOT EXISTS idx_rule_categories_parent ON rule_categories (parent_id);
