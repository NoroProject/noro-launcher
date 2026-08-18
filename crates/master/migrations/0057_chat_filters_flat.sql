-- Пороги фильтров чата — колонками, а не JSONB.
--
-- 0053 завела `config JSONB`, но ни мастер, ни агент так и не научились его
-- читать: обе стороны ждали плоских полей, просто под разными именами. В итоге
-- `GET /api/agent/chat-filters` и админка падали с «column does not exist», а
-- автомодерация не работала ни в одном режиме.
--
-- Имена берём те, что уже понимает агент (`FilterConfig`): они не привязаны к
-- виду фильтра. `caps_ratio` в строке рекламного фильтра не значит ничего, а
-- `threshold` — значит, и это единственная таблица на все четыре вида.
--
-- `rule_code` (пункт свода, по которому наказывать) добавлен в 0054 и остаётся.

ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS whitelist TEXT[] NOT NULL DEFAULT '{}';
ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS words TEXT[] NOT NULL DEFAULT '{}';
ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS threshold DOUBLE PRECISION NOT NULL DEFAULT 0.6;
ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS min_length INT NOT NULL DEFAULT 6;
ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS max_messages INT NOT NULL DEFAULT 3;
ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS window_secs INT NOT NULL DEFAULT 4;
ALTER TABLE chat_filters ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

-- Перенос того, что успели настроить через JSONB. Ключи те же, что в засеве
-- 0053, поэтому перенос точный; чего в конфиге не было — остаётся умолчанием.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1 FROM information_schema.columns
        WHERE table_name = 'chat_filters' AND column_name = 'config'
    ) THEN
        UPDATE chat_filters SET
            whitelist = COALESCE(
                ARRAY(SELECT jsonb_array_elements_text(config -> 'whitelist')), whitelist),
            words = COALESCE(
                ARRAY(SELECT jsonb_array_elements_text(config -> 'words')), words),
            threshold = COALESCE((config ->> 'threshold')::DOUBLE PRECISION, threshold),
            min_length = COALESCE((config ->> 'min_length')::INT, min_length),
            max_messages = COALESCE((config ->> 'max_messages')::INT, max_messages),
            window_secs = COALESCE((config ->> 'window_secs')::INT, window_secs);
        ALTER TABLE chat_filters DROP COLUMN config;
    END IF;
END $$;

-- Словарь мата пустой по умолчанию. Засев 0053 клал туда `badword` — слово из
-- примера, по которому фильтр срабатывал бы на живом чате.
UPDATE chat_filters SET words = '{}' WHERE filter_type = 'word' AND words = ARRAY['badword'];

-- Те же имена, что у фильтров: записи о срабатываниях — их продолжение, и
-- расходиться в словаре двум соседним таблицам незачем. Таблица заведена в
-- 0054 и на момент переименования пуста.
--
-- Под проверкой, потому что RENAME не имеет формы IF EXISTS и на второй заход
-- падает. Мастер прогоняет миграции при каждом старте, и невыполнимый файл
-- посреди списка означает, что сервер не поднимется.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'automod_triggers' AND column_name = 'rule_type') THEN
        ALTER TABLE automod_triggers RENAME COLUMN rule_type TO filter_type;
    END IF;
    IF EXISTS (SELECT 1 FROM information_schema.columns
               WHERE table_name = 'automod_triggers' AND column_name = 'action') THEN
        ALTER TABLE automod_triggers RENAME COLUMN action TO mode;
    END IF;
END $$;

-- Пункт свода каждому фильтру.
--
-- Без него автомодерация выдаёт наказание, не ограниченное ничем: рамки берутся
-- из санкций указанного правила, а при пустом коде `check_limits` пропускает
-- любой срок — см. `api/agent/punish.rs`. Именно эта связка и описана в плане
-- как единственная защита от сервера, наказывающего сам.
--
-- Только там, где код ещё не проставлен, и только если такое правило есть:
-- свод правится в админке, и навязывать свои коды поверх чужих нельзя.
UPDATE chat_filters f SET rule_code = v.code
FROM (VALUES
        ('ad',    '1.4'),   -- Реклама сторонних проектов
        ('word',  '2.2'),   -- Мат в общем чате
        ('caps',  '2.1'),   -- Флуд и спам
        ('flood', '2.1')    -- Флуд и спам
     ) AS v(filter_type, code)
WHERE f.filter_type = v.filter_type
  AND f.rule_code IS NULL
  AND EXISTS (SELECT 1 FROM rules r WHERE r.code = v.code);
