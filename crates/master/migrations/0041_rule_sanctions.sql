-- Наказание за правило — не одно фиксированное число, а набор допустимых:
-- «первое нарушение — мут 30м–2ч», «повторное — бан 7д–30д». Модератор
-- выбирает из этого набора и двигает срок внутри рамок, а выйти за них может
-- только тот, кому выдан `noro.mod.punish.bypass`.

CREATE TABLE IF NOT EXISTS rule_sanctions (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id     UUID NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
    kind        TEXT NOT NULL CHECK (kind IN ('warn', 'mute', 'ban', 'server_ban')),
    -- Пояснение к варианту: «первое нарушение», «в особо грубой форме».
    label       TEXT NOT NULL DEFAULT '',
    -- Границы срока в минутах. NULL снизу — без минимума, NULL сверху —
    -- разрешено вплоть до «навсегда».
    min_minutes BIGINT CHECK (min_minutes > 0),
    max_minutes BIGINT CHECK (max_minutes > 0),
    sort_order  INT NOT NULL DEFAULT 0,
    CHECK (min_minutes IS NULL OR max_minutes IS NULL OR max_minutes >= min_minutes)
);

CREATE INDEX IF NOT EXISTS idx_rule_sanctions_rule ON rule_sanctions (rule_id);

-- Единственная рекомендация, что была у правила, становится первым вариантом.
INSERT INTO rule_sanctions (rule_id, kind, min_minutes, max_minutes)
SELECT id, punishment_kind, punishment_minutes, punishment_minutes
  FROM rules
 WHERE punishment_kind IS NOT NULL;

ALTER TABLE rules DROP COLUMN IF EXISTS punishment_kind;
ALTER TABLE rules DROP COLUMN IF EXISTS punishment_minutes;

-- Наказание ссылается на правило. Код дублируется снимком: правило переименуют
-- или удалят, а в разборе через год должно остаться видно, за что наказали.
ALTER TABLE punishments ADD COLUMN IF NOT EXISTS rule_id UUID REFERENCES rules(id) ON DELETE SET NULL;
ALTER TABLE punishments ADD COLUMN IF NOT EXISTS rule_code TEXT;
