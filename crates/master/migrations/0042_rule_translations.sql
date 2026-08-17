-- Переводы содержимого свода.
--
-- Текст правила пишет команда, а не разработчик, поэтому в ftl-каталог его не
-- положить: там ключи интерфейса. Базовая формулировка остаётся в `rules`
-- (она же фолбэк, если перевода нет), а переводы лежат рядом по локалям —
-- иначе переключение языка показывало бы пустоту вместо ещё не переведённого
-- пункта.

CREATE TABLE IF NOT EXISTS rule_translations (
    rule_id     UUID NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
    locale      TEXT NOT NULL,
    title       VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (rule_id, locale)
);

CREATE TABLE IF NOT EXISTS rule_category_translations (
    category_id UUID NOT NULL REFERENCES rule_categories(id) ON DELETE CASCADE,
    locale      TEXT NOT NULL,
    name        VARCHAR(255) NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (category_id, locale)
);

-- Пояснение к варианту наказания («первое нарушение») тоже читает игрок.
CREATE TABLE IF NOT EXISTS rule_sanction_translations (
    sanction_id UUID NOT NULL REFERENCES rule_sanctions(id) ON DELETE CASCADE,
    locale      TEXT NOT NULL,
    label       TEXT NOT NULL DEFAULT '',
    PRIMARY KEY (sanction_id, locale)
);
