-- Каталоги локализации в формате Fluent (.ftl).
-- Текст лежит прямо в БД: каталоги маленькие (единицы КБ) и меняются целиком,
-- поэтому content-addressed стор здесь только мешал бы — понадобилась бы
-- сборка мусора при каждом сохранении из админки.

CREATE TABLE IF NOT EXISTS translations (
    locale     TEXT PRIMARY KEY,
    ftl        TEXT NOT NULL,
    sha1       TEXT NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by UUID REFERENCES users(id)
);
