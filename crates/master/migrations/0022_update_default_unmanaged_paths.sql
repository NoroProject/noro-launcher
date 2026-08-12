-- Изменение дефолтного значения столбца unmanaged_paths для новых сборок
ALTER TABLE builds
ALTER COLUMN unmanaged_paths SET DEFAULT '["saves/","screenshots/","options.txt","optionsof.txt","optionsshaders.txt","logs/","crash-reports/","xaero*","config/xaero*","xaerominimap*","xaeroworldmap*"]'::jsonb;

-- Обновление уже существующих сборок в БД на проде:
-- Дополняем их unmanaged_paths масками xaero* и config/xaero*, если их там ещё нет
UPDATE builds
SET unmanaged_paths = unmanaged_paths || '["xaero*","config/xaero*","xaerominimap*","xaeroworldmap*","optionsof.txt","optionsshaders.txt"]'::jsonb
WHERE NOT (unmanaged_paths @> '["xaero*"]'::jsonb);
